use std::fmt::Display;

use anyhow::Result;
use penumbra_community_pool::params::CommunityPoolParameters;
use penumbra_dex::DexParameters;
use penumbra_distributions::params::DistributionsParameters;
use penumbra_fee::FeeParameters;
use penumbra_funding::params::FundingParameters;
use penumbra_governance::{
    params::GovernanceParameters, proposal::ChangedAppParameters, tally::Ratio,
};
use penumbra_ibc::params::IBCParameters;
use penumbra_sct::params::SctParameters;
use penumbra_shielded_pool::params::ShieldedPoolParameters;
use penumbra_stake::params::StakeParameters;

use super::AppParameters;

// The checks below validate that a parameter change is valid, since some parameter settings or
// combinations are nonsensical and should be rejected outright, regardless of governance.

#[deny(unused)] // We want to be really careful here to not examine fields!
impl AppParameters {
    pub fn check_valid_update(&self, new: &AppParameters) -> Result<()> {
        new.check_valid()?;
        // TODO: move the checks below into their respective components.
        // Tracked by #3593
        let AppParameters {
            chain_id,
            community_pool_params:
                CommunityPoolParameters {
                    community_pool_spend_proposals_enabled: _,
                },
            distributions_params:
                DistributionsParameters {
                    staking_issuance_per_block: _,
                },
            fee_params: FeeParameters {
                fixed_gas_prices: _,
            },
            funding_params: FundingParameters {},
            governance_params:
                GovernanceParameters {
                    proposal_voting_blocks: _,
                    proposal_deposit_amount: _,
                    proposal_valid_quorum,
                    proposal_pass_threshold,
                    proposal_slash_threshold,
                    community_pool_is_frozen: _,
                },
            ibc_params:
                IBCParameters {
                    ibc_enabled: _,
                    inbound_ics20_transfers_enabled: _,
                    outbound_ics20_transfers_enabled: _,
                },
            sct_params: SctParameters { epoch_duration },
            shielded_pool_params:
                ShieldedPoolParameters {
                    fixed_fmd_params: _,
                },
            stake_params:
                StakeParameters {
                    active_validator_limit,
                    base_reward_rate: _,
                    slashing_penalty_misbehavior: _,
                    slashing_penalty_downtime: _,
                    signed_blocks_window_len,
                    missed_blocks_maximum: _,
                    min_validator_stake: _,
                    unbonding_delay: _,
                },
            dex_params:
                DexParameters {
                    is_enabled: _,
                    fixed_candidates: _,
                    max_hops: _,
                    max_positions_per_pair: _,
                },
            // IMPORTANT: Don't use `..` here! We want to ensure every single field is verified!
        } = self;

        // Ensure that certain parameters are not changed by the update:
        check_invariant([(chain_id, &new.chain_id, "chain ID")])?;
        check_invariant([
            (
                epoch_duration,
                &new.sct_params.epoch_duration,
                "epoch duration",
            ),
            (
                active_validator_limit,
                &new.stake_params.active_validator_limit,
                "active validator limit",
            ),
            (
                signed_blocks_window_len,
                &new.stake_params.signed_blocks_window_len,
                "signed blocks window length",
            ),
        ])?;
        check_invariant([
            (
                proposal_valid_quorum,
                &new.governance_params.proposal_valid_quorum,
                "proposal valid quorum",
            ),
            (
                proposal_pass_threshold,
                &new.governance_params.proposal_pass_threshold,
                "proposal pass threshold",
            ),
            (
                proposal_slash_threshold,
                &new.governance_params.proposal_slash_threshold,
                "proposal slash threshold",
            ),
        ])?;

        Ok(())
    }

    pub fn check_valid(&self) -> Result<()> {
        let AppParameters {
            chain_id,
            community_pool_params:
                CommunityPoolParameters {
                    community_pool_spend_proposals_enabled: _,
                },
            distributions_params:
                DistributionsParameters {
                    staking_issuance_per_block: _,
                },
            fee_params: FeeParameters {
                fixed_gas_prices: _,
            },
            funding_params: FundingParameters {},
            governance_params:
                GovernanceParameters {
                    proposal_voting_blocks,
                    proposal_deposit_amount,
                    proposal_valid_quorum,
                    proposal_pass_threshold,
                    proposal_slash_threshold,
                    community_pool_is_frozen: _,
                },
            ibc_params:
                IBCParameters {
                    ibc_enabled,
                    inbound_ics20_transfers_enabled,
                    outbound_ics20_transfers_enabled,
                },
            sct_params: SctParameters { epoch_duration },
            shielded_pool_params:
                ShieldedPoolParameters {
                    fixed_fmd_params: _,
                },
            stake_params:
                StakeParameters {
                    active_validator_limit,
                    base_reward_rate,
                    slashing_penalty_misbehavior,
                    slashing_penalty_downtime,
                    signed_blocks_window_len,
                    missed_blocks_maximum,
                    min_validator_stake,
                    unbonding_delay,
                },
            dex_params:
                DexParameters {
                    is_enabled: _,
                    fixed_candidates: _,
                    max_hops: _,
                    max_positions_per_pair: _,
                },
            // IMPORTANT: Don't use `..` here! We want to ensure every single field is verified!
        } = self;

        check_all([
            (!chain_id.is_empty(), "chain ID must be a non-empty string"),
            (
                *epoch_duration >= 1,
                "epoch duration must be at least one block",
            ),
            (
                *unbonding_delay >= epoch_duration * 2 + 1,
                "unbonding must take at least two epochs",
            ),
            (
                *active_validator_limit > 3,
                "active validator limit must be at least 4",
            ),
            (
                *base_reward_rate >= 1,
                "base reward rate must be at least 1 basis point",
            ),
            (
                *slashing_penalty_misbehavior >= 1,
                "slashing penalty (misbehavior) must be at least 1 basis point",
            ),
            (
                *slashing_penalty_misbehavior <= 100_000_000,
                "slashing penalty (misbehavior) must be at most 10,000 basis points^2",
            ),
            (
                *slashing_penalty_downtime >= 1,
                "slashing penalty (downtime) must be at least 1 basis point",
            ),
            (
                *slashing_penalty_downtime <= 100_000_000,
                "slashing penalty (downtime) must be at most 10,000 basis points^2",
            ),
            (
                *signed_blocks_window_len >= 2,
                "signed blocks window length must be at least 2",
            ),
            (
                *missed_blocks_maximum >= 1,
                "missed blocks maximum must be at least 1",
            ),
            (
                (!*inbound_ics20_transfers_enabled && !*outbound_ics20_transfers_enabled)
                    || *ibc_enabled,
                "IBC must be enabled if either inbound or outbound ICS20 transfers are enabled",
            ),
            (
                *proposal_voting_blocks >= 1,
                "proposal voting blocks must be at least 1",
            ),
            (
                *proposal_deposit_amount >= 1u64.into(),
                "proposal deposit amount must be at least 1",
            ),
            (
                *proposal_valid_quorum > Ratio::new(0, 1),
                "proposal valid quorum must be greater than 0",
            ),
            (
                *proposal_pass_threshold >= Ratio::new(1, 2),
                "proposal pass threshold must be greater than or equal to 1/2",
            ),
            (
                *proposal_slash_threshold > Ratio::new(1, 2),
                "proposal slash threshold must be greater than 1/2",
            ),
            (
                *min_validator_stake >= 1_000_000u128.into(),
                "the minimum validator stake must be at least 1penumbra",
            ),
            // TODO(erwan): add a `max_positions_per_pair` check
        ])
    }

    /// Converts an `AppParameters` instance to a complete `ChangedAppParameters`.
    pub fn as_changed_params(&self) -> ChangedAppParameters {
        ChangedAppParameters {
            community_pool_params: Some(self.community_pool_params.clone()),
            distributions_params: Some(self.distributions_params.clone()),
            fee_params: Some(self.fee_params.clone()),
            funding_params: Some(self.funding_params.clone()),
            governance_params: Some(self.governance_params.clone()),
            ibc_params: Some(self.ibc_params.clone()),
            shielded_pool_params: Some(self.shielded_pool_params.clone()),
            sct_params: Some(self.sct_params.clone()),
            stake_params: Some(self.stake_params.clone()),
            dex_params: Some(self.dex_params.clone()),
        }
    }

    /// Converts a sparse ChangedAppParameters into a complete AppParameters, filling
    /// in any `None` values from the old parameters.
    ///
    /// Throws an error if `old` is `None` and any of the component parameters in `new` are
    /// `None`, i.e. all fields in `new` must be `Some` if `old` is not provided.
    pub fn from_changed_params(
        new: &ChangedAppParameters,
        old: Option<&AppParameters>,
    ) -> Result<AppParameters> {
        if old.is_none()
            && (new.community_pool_params.is_none()
                || new.distributions_params.is_none()
                || new.fee_params.is_none()
                || new.funding_params.is_none()
                || new.governance_params.is_none()
                || new.ibc_params.is_none()
                || new.sct_params.is_none()
                || new.shielded_pool_params.is_none()
                || new.stake_params.is_none())
        {
            anyhow::bail!("all parameters must be specified if no old parameters are provided");
        }

        Ok(AppParameters {
            // TODO(erwan): we are momentarily not supporting chain_id changes
            // until the IBC host chain changes land.
            // See: https://github.com/penumbra-zone/penumbra/issues/3617#issuecomment-1917708221
            chain_id: old
                .expect("old should be set if new has any None values")
                .chain_id
                .clone(),
            community_pool_params: new.community_pool_params.clone().unwrap_or_else(|| {
                old.expect("old should be set if new has any None values")
                    .community_pool_params
                    .clone()
            }),
            distributions_params: new.distributions_params.clone().unwrap_or_else(|| {
                old.expect("old should be set if new has any None values")
                    .distributions_params
                    .clone()
            }),
            fee_params: new.fee_params.clone().unwrap_or_else(|| {
                old.expect("old should be set if new has any None values")
                    .fee_params
                    .clone()
            }),
            funding_params: new.funding_params.clone().unwrap_or_else(|| {
                old.expect("old should be set if new has any None values")
                    .funding_params
                    .clone()
            }),
            governance_params: new.governance_params.clone().unwrap_or_else(|| {
                old.expect("old should be set if new has any None values")
                    .governance_params
                    .clone()
            }),
            ibc_params: new.ibc_params.clone().unwrap_or_else(|| {
                old.expect("old should be set if new has any None values")
                    .ibc_params
                    .clone()
            }),
            sct_params: new.sct_params.clone().unwrap_or_else(|| {
                old.expect("old should be set if new has any None values")
                    .sct_params
                    .clone()
            }),
            shielded_pool_params: new.shielded_pool_params.clone().unwrap_or_else(|| {
                old.expect("old should be set if new has any None values")
                    .shielded_pool_params
                    .clone()
            }),
            stake_params: new.stake_params.clone().unwrap_or_else(|| {
                old.expect("old should be set if new has any None values")
                    .stake_params
                    .clone()
            }),
            dex_params: new.dex_params.clone().unwrap_or_else(|| {
                old.expect("old should be set if new has any None values")
                    .dex_params
                    .clone()
            }),
        })
    }
}

/// Ensure all of the booleans are true, and if any are false, generate an error describing which
/// failed, based on the provided descriptions.
fn check_all<'a>(checks: impl IntoIterator<Item = (bool, impl Display + 'a)>) -> Result<()> {
    let failed_because = checks
        .into_iter()
        .filter_map(|(ok, description)| {
            if !ok {
                Some(description.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if !failed_because.is_empty() {
        anyhow::bail!("invalid chain parameters: {}", failed_because.join(", "));
    }

    Ok(())
}

/// Ensure that all of the provided pairs of values are equal, and if any are not, generate an error
/// stating that the varying names can't be changed.
fn check_invariant<'a, T: Eq + 'a>(
    sides: impl IntoIterator<Item = (&'a T, &'a T, impl Display + 'a)>,
) -> Result<()> {
    check_all(
        sides
            .into_iter()
            .map(|(old, new, name)| ((*old == *new), format!("{name} can't be changed"))),
    )
}
