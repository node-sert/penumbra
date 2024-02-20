use penumbra_distributions::component::StateReadExt as _;
use penumbra_sct::{component::clock::EpochRead, epoch::Epoch};
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::StreamExt;
use penumbra_asset::STAKING_TOKEN_ASSET_ID;

use cnidarium::StateWrite;
use penumbra_num::{fixpoint::U128x128, Amount};
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_shielded_pool::component::{SupplyRead, SupplyWrite};
use tendermint::validator::Update;
use tendermint::PublicKey;
use tokio::task::JoinSet;
use tracing::instrument;

use crate::state_key;
use crate::{
    component::{
        stake::{ConsensusUpdateWrite, InternalStakingData, RateDataWrite},
        validator_handler::{ValidatorDataRead, ValidatorDataWrite, ValidatorManager},
        SlashingData, FP_SCALING_FACTOR,
    },
    rate::BaseRateData,
    validator, CurrentConsensusKeys, DelegationToken, FundingStreams, IdentityKey, Penalty,
    StateReadExt,
};

use super::{ConsensusIndexRead, StateWriteExt};

#[async_trait]
pub trait EpochHandler: StateWriteExt + ConsensusIndexRead {
    #[instrument(skip(self, epoch_to_end), fields(index = epoch_to_end.index))]
    /// Process the end of an epoch for the staking component.
    async fn end_epoch(&mut self, epoch_to_end: Epoch) -> Result<()> {
        // Collect all the delegation changes that occurred in the epoch we are ending.
        let mut delegations_by_validator = BTreeMap::<IdentityKey, Amount>::new();
        let mut undelegations_by_validator = BTreeMap::<IdentityKey, Amount>::new();

        let end_height = self.get_block_height().await?;
        let mut num_delegations = 0usize;
        let mut num_undelegations = 0usize;

        for height in epoch_to_end.start_height..=end_height {
            let changes = self
                .get_delegation_changes(
                    height
                        .try_into()
                        .context("should be able to convert u64 into block height")?,
                )
                .await?;

            num_delegations = num_delegations.saturating_add(changes.delegations.len());
            num_undelegations = num_undelegations.saturating_add(changes.undelegations.len());

            for d in changes.delegations {
                let validator_identity = d.validator_identity.clone();
                delegations_by_validator
                    .entry(validator_identity)
                    .or_default()
                    .saturating_add(&d.delegation_amount);
            }
            for u in changes.undelegations {
                let validator_identity = u.validator_identity.clone();
                undelegations_by_validator
                    .entry(validator_identity)
                    .or_default()
                    .saturating_add(&u.delegation_amount);
            }
        }

        tracing::debug!(
            num_delegations,
            num_undelegations,
            epoch_start = epoch_to_end.start_height,
            epoch_end = end_height,
            epoch_index = epoch_to_end.index,
            "collected delegation changes for the epoch"
        );

        // We are transitioning to the next epoch, so the "current" base rate in
        // the state is now the previous base rate.
        let prev_base_rate = self.get_current_base_rate().await?;

        tracing::debug!(
            "fetching the issuance budget for this epoch from the distributions component"
        );
        // Fetch the issuance budget for the epoch we are ending.
        let issuance_budget_for_epoch = self
            .get_staking_token_issuance_for_epoch()
            .expect("issuance budget is always set by the distributions component");

        // Compute the base reward rate for the upcoming epoch based on the total amount
        // of active stake and the issuance budget given to us by the distribution component.
        let total_active_stake_previous_epoch = self.total_active_stake().await?;
        tracing::debug!(
            ?total_active_stake_previous_epoch,
            ?issuance_budget_for_epoch,
            "computing base rate for the upcoming epoch"
        );

        let base_reward_rate =
            U128x128::ratio(issuance_budget_for_epoch, total_active_stake_previous_epoch)
                .expect("total active stake is nonzero");
        let base_reward_rate: Amount = (base_reward_rate * *FP_SCALING_FACTOR)
            .expect("base reward rate is around one")
            .round_down()
            .try_into()
            .expect("rounded to an integral value");
        tracing::debug!(%base_reward_rate, "base reward rate for the upcoming epoch");

        let next_base_rate = prev_base_rate.next_epoch(base_reward_rate);
        tracing::debug!(
            ?prev_base_rate,
            ?next_base_rate,
            ?base_reward_rate,
            ?total_active_stake_previous_epoch,
            ?issuance_budget_for_epoch,
            "calculated base rate for the upcoming epoch"
        );

        // Set the next base rate as the new "current" base rate.
        self.set_base_rate(next_base_rate.clone());
        // We cache the previous base rate in the state, so that other components
        // can use it in their end-epoch procesisng (e.g. funding for staking rewards).
        self.set_prev_base_rate(prev_base_rate.clone());

        let mut funding_queue: Vec<(IdentityKey, FundingStreams, Amount)> = Vec::new();
        let mut validator_stream = self.consensus_set_stream()?;

        while let Some(validator_identity) = validator_stream.next().await {
            let validator_identity = validator_identity?;
            let total_delegations = delegations_by_validator
                .remove(&validator_identity)
                .unwrap_or_else(Amount::zero);

            let total_undelegations = undelegations_by_validator
                .remove(&validator_identity)
                .unwrap_or_else(Amount::zero);

            self.process_validator(
                validator_identity,
                epoch_to_end,
                next_base_rate.clone(),
                total_delegations,
                total_undelegations,
            )
            .await
            .map_err(|e| {
                tracing::error!(
                    ?e,
                    ?validator_identity,
                    "failed to process validator's end-epoch"
                );
                e
            })?
            .map(|(identity, funding_streams, delegation_token_supply)| {
                funding_queue.push((identity, funding_streams, delegation_token_supply))
            });
        }

        // We have collected the funding streams for all validators, so we can now
        // record them for the funding component to process.
        self.queue_staking_rewards(funding_queue);

        // Now that the consensus set voting power has been calculated, we can select the
        // top N validators to be active for the next epoch.
        self.set_active_and_inactive_validators().await?;
        Ok(())
    }

    async fn process_validator(
        &mut self,
        validator_identity: IdentityKey,
        epoch_to_end: Epoch,
        next_base_rate: BaseRateData,
        total_delegations: Amount,
        total_undelegations: Amount,
    ) -> Result<Option<(IdentityKey, FundingStreams, Amount)>> {
        let min_validator_stake = self.get_stake_params().await?.min_validator_stake;

        let validator = self.get_validator_definition(&validator_identity).await?.ok_or_else(|| {
            anyhow::anyhow!("validator (identity={}) is in consensus index but its definition was not found in the JMT", &validator_identity)
        })?;

        // Grab the current validator state.
        let validator_state = self
            .get_validator_state(&validator.identity_key)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("validator (identity={}) is in consensus index but its state was not found in the JMT", &validator.identity_key)
            })?;

        // We are transitioning to the next epoch, so the "current" validator
        // rate in the state is now the previous validator rate.
        let prev_validator_rate = self
            .get_validator_rate(&validator.identity_key)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("validator (identity={}) is in consensus index but its rate data was not found in the JMT", &validator.identity_key)
            })?;

        // First, apply any penalty recorded in the epoch we are ending.
        let penalty = self
            .get_penalty_in_epoch(&validator.identity_key, epoch_to_end.index)
            .await
            .unwrap_or(Penalty::from_percent(0));
        let prev_validator_rate_with_penalty = prev_validator_rate.slash(penalty);

        self.set_prev_validator_rate(
            &validator.identity_key,
            prev_validator_rate_with_penalty.clone(),
        );

        // Then compute the next validator rate, accounting for funding streams and validator state.
        let next_validator_rate = prev_validator_rate_with_penalty.next_epoch(
            &next_base_rate,
            validator.funding_streams.as_ref(),
            &validator_state,
        );

        // In theory, the maximum amount of delegation tokens is the total supply of staking tokens.
        // In practice, this is unlikely to happen, but even if it did, we anticipate that the total
        // supply of staking token is << 10^32 (2^107) tokens with a unit denomination of 10^6 (2^20),
        // so there should be ample room to cast this to an i128.
        let delegation_delta =
            (total_delegations.value() as i128) - (total_undelegations.value() as i128);

        tracing::debug!(
            validator = ?validator.identity_key,
            ?total_delegations,
            ?total_undelegations,
            delegation_delta,
            "net delegation change for validator's pool for the epoch"
        );

        // Delegations and undelegations created in the previous epoch were created
        // with the prev_validator_rate.  To compute the staking delta, we need to take
        // an absolute value and then re-apply the sign, since .unbonded_amount operates
        // on unsigned values.
        let absolute_delegation_change = Amount::from(delegation_delta.unsigned_abs());
        let absolute_unbonded_amount =
            prev_validator_rate.unbonded_amount(absolute_delegation_change);

        let delegation_token_id = DelegationToken::from(&validator.identity_key).id();

        // Staking tokens are being delegated, so the staking token supply decreases and
        // the delegation token supply increases.
        if delegation_delta > 0 {
            tracing::debug!(
                validator = ?validator.identity_key,
                "staking tokens are being delegated, so the staking token supply decreases and the delegation token supply increases");
            self.decrease_token_supply(&STAKING_TOKEN_ASSET_ID, absolute_unbonded_amount)
                .await
                .with_context(|| {
                    format!(
                        "failed to decrease staking token supply by {}",
                        absolute_unbonded_amount
                    )
                })?;
            self.increase_token_supply(&delegation_token_id, absolute_delegation_change)
                .await
                .with_context(|| {
                    format!(
                        "failed to increase delegation token supply by {}",
                        absolute_delegation_change
                    )
                })?;
        } else if delegation_delta < 0 {
            tracing::debug!(
                validator = ?validator.identity_key,
                "staking tokens are being undelegated, so the staking token supply increases and the delegation token supply decreases");
            // Vice-versa: staking tokens are being undelegated, so the staking token supply
            // increases and the delegation token supply decreases.
            self.increase_token_supply(&STAKING_TOKEN_ASSET_ID, absolute_unbonded_amount)
                .await
                .with_context(|| {
                    format!(
                        "failed to increase staking token supply by {}",
                        absolute_unbonded_amount
                    )
                })?;
            self.decrease_token_supply(&delegation_token_id, absolute_delegation_change)
                .await
                .with_context(|| {
                    format!(
                        "failed to decrease delegation token supply by {}",
                        absolute_delegation_change
                    )
                })?;
        } else {
            tracing::debug!(
                validator = ?validator.identity_key,
                "no change in delegation, no change in token supply")
        }

        // Get the updated delegation token supply for use calculating voting power.
        let delegation_token_supply = self
            .token_supply(&delegation_token_id)
            .await?
            .expect("delegation token should be known");

        // Calculate the voting power in the newly beginning epoch
        let voting_power = next_validator_rate.voting_power(delegation_token_supply);

        tracing::debug!(
            validator = ?validator.identity_key,
            validator_delegation_pool = ?delegation_token_supply,
            validator_power = ?voting_power,
            "calculated validator's voting power for the upcoming epoch"
        );

        // Update the state of the validator within the validator set
        // with the newly starting epoch's calculated voting rate and power.
        self.set_validator_rate_data(&validator.identity_key, next_validator_rate.clone());
        self.set_validator_power(&validator.identity_key, voting_power)?;

        // The epoch is ending, so we check if this validator was active and if so
        // we queue its [`FundingStreams`] for processing by the funding component.
        let reward_queue_entry = if validator_state == validator::State::Active {
            // Here we collect funding data to create a record that the funding component
            // can "pull". We do this because by the time the funding component is executed
            // the validator set has possibly changed (e.g. a new validator enter the active
            // set).
            Some((
                validator.identity_key.clone(),
                validator.funding_streams.clone(),
                delegation_token_supply,
            ))
        } else {
            None
        };

        // We want to know if the validator has enough stake to remain in the consensus set.
        // In order to do this, we need to know what is the size of the validator's delegation
        // pool in terms of staking tokens (i.e. the unbonded amount).
        let delegation_token_denom = DelegationToken::from(&validator.identity_key).denom();
        let validator_unbonded_amount =
            next_validator_rate.unbonded_amount(delegation_token_supply);

        tracing::debug!(
            validator_identity = %validator.identity_key,
            validator_delegation_pool = ?delegation_token_supply,
            validator_unbonded_amount = ?validator_unbonded_amount,
            "calculated validator's unbonded amount for the upcoming epoch"
        );

        if validator_unbonded_amount < min_validator_stake {
            tracing::debug!(
                validator_identity = %validator.identity_key,
                validator_unbonded_amount = ?validator_unbonded_amount,
                min_validator_stake = ?min_validator_stake,
                "validator's unbonded amount is below the minimum stake threshold, transitioning to defined"
            );
            self.set_validator_state(&validator.identity_key, validator::State::Defined)
                .await?;
        }

        tracing::debug!(validator_identity = %validator.identity_key,
            previous_epoch_validator_rate= ?prev_validator_rate,
            next_epoch_validator_rate = ?next_validator_rate,
            delegation_denom = ?delegation_token_denom,
            ?delegation_token_supply,
            "validator's end-epoch has been processed");

        self.process_validator_pool_state(&validator.identity_key, epoch_to_end)
            .await.map_err(|e| {
                tracing::error!(?e, validator_identity = %validator.identity_key, "failed to process validator pool state");
                e
            })?;

        Ok(reward_queue_entry)
    }

    /// Called during `end_epoch`. Will perform state transitions to validators based
    /// on changes to voting power that occurred in this epoch.
    async fn set_active_and_inactive_validators(&mut self) -> Result<()> {
        // A list of all active and inactive validators, with nonzero voting power.
        let mut validators_by_power = Vec::new();
        // A list of validators with zero power, who must be inactive.
        let mut zero_power = Vec::new();

        let mut validator_identity_stream = self.consensus_set_stream()?;
        while let Some(identity_key) = validator_identity_stream.next().await {
            let identity_key = identity_key?;
            let state = self
                .get_validator_state(&identity_key)
                .await?
                .context("should be able to fetch validator state")?;
            let power = self
                .get_validator_power(&identity_key)
                .await?
                .unwrap_or_default();
            if matches!(state, validator::State::Active | validator::State::Inactive) {
                if power == Amount::zero() {
                    zero_power.push((identity_key, power));
                } else {
                    validators_by_power.push((identity_key, power));
                }
            }
        }

        // Sort by voting power descending.
        validators_by_power.sort_by(|a, b| b.1.cmp(&a.1));

        // The top `limit` validators with nonzero power become active.
        // All other validators become inactive.
        let limit = self.get_stake_params().await?.active_validator_limit as usize;
        let active = validators_by_power.iter().take(limit);
        let inactive = validators_by_power
            .iter()
            .skip(limit)
            .chain(zero_power.iter());

        for (v, _) in active {
            self.set_validator_state(v, validator::State::Active)
                .await?;
        }
        for (v, _) in inactive {
            self.set_validator_state(v, validator::State::Inactive)
                .await?;
        }

        Ok(())
    }

    /// Materializes the entire current validator set as a CometBFT update.
    ///
    /// This re-defines all validators every time, to simplify the code compared to
    /// trying to track delta updates.
    #[instrument(skip(self))]
    async fn build_cometbft_validator_updates(&mut self) -> Result<()> {
        let current_consensus_keys: CurrentConsensusKeys = self
            .get(state_key::consensus_update::consensus_keys())
            .await?
            .expect("current consensus keys must be present");
        let current_consensus_keys = current_consensus_keys
            .consensus_keys
            .into_iter()
            .collect::<BTreeSet<_>>();

        let mut voting_power_by_consensus_key = BTreeMap::<PublicKey, Amount>::new();

        // First, build a mapping of consensus key to voting power for all known validators.

        // Using a JoinSet, run each validator's state queries concurrently.
        let mut js: JoinSet<std::prelude::v1::Result<(PublicKey, Amount), anyhow::Error>> =
            JoinSet::new();
        let mut validator_identity_stream = self.consensus_set_stream()?;
        while let Some(identity_key) = validator_identity_stream.next().await {
            let identity_key = identity_key?;
            let state = self.get_validator_state(&identity_key);
            let power = self.get_validator_power(&identity_key);
            let consensus_key = self.fetch_validator_consensus_key(&identity_key);
            js.spawn(async move {
                let state = state
                    .await?
                    .expect("every known validator must have a recorded state");
                // Compute the effective power of this validator; this is the
                // validator power, clamped to zero for all non-Active validators.
                let effective_power = if matches!(state, validator::State::Active) {
                    power
                        .await?
                        .expect("every active validator must have a recorded power")
                } else {
                    Amount::zero()
                };

                let consensus_key = consensus_key
                    .await?
                    .expect("every known validator must have a recorded consensus key");

                anyhow::Ok((consensus_key, effective_power))
            });
        }
        // Now collect the computed results into the lookup table.
        while let Some(pair) = js.join_next().await.transpose()? {
            let (consensus_key, effective_power) = pair?;
            voting_power_by_consensus_key.insert(consensus_key, effective_power);
        }

        // Next, filter that mapping to exclude any zero-power validators, UNLESS they
        // were already known to CometBFT.
        voting_power_by_consensus_key.retain(|consensus_key, voting_power| {
            *voting_power > Amount::zero() || current_consensus_keys.contains(consensus_key)
        });

        // Finally, tell tendermint to delete any known consensus keys not otherwise updated
        for ck in current_consensus_keys.iter() {
            voting_power_by_consensus_key
                .entry(*ck)
                .or_insert(Amount::zero());
        }

        // Save the validator updates to send to Tendermint.
        let tendermint_validator_updates = voting_power_by_consensus_key
            .iter()
            .map(|(consensus_key, power)| {
                Ok(Update {
                    pub_key: *consensus_key,
                    // Validator voting power is measured in units of staking tokens,
                    // at time, CometBFT has an upper limit of around 2^60 - 1.
                    // This means that there is an upper bound on the maximum possible issuance
                    // at around 10^12 units of staking tokens.
                    power: ((*power).value() as u64).try_into()?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        self.put_cometbft_validator_updates(tendermint_validator_updates);

        // Record the new consensus keys we will have told tendermint about.
        let updated_consensus_keys = CurrentConsensusKeys {
            consensus_keys: voting_power_by_consensus_key
                .iter()
                .filter_map(|(consensus_key, power)| {
                    if *power != Amount::zero() {
                        Some(*consensus_key)
                    } else {
                        None
                    }
                })
                .collect(),
        };
        tracing::debug!(?updated_consensus_keys);
        self.put(
            state_key::consensus_update::consensus_keys().to_owned(),
            updated_consensus_keys,
        );

        Ok(())
    }
}

impl<T: StateWrite + ConsensusIndexRead + ?Sized> EpochHandler for T {}
