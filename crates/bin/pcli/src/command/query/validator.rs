use std::{fs::File, io::Write};

use anyhow::{Context, Result};
use colored::Colorize;
use comfy_table::{presets, Table};
use futures::TryStreamExt;
use penumbra_num::Amount;
use penumbra_proto::core::component::stake::v1::{
    query_service_client::QueryServiceClient as StakeQueryServiceClient, ValidatorInfoRequest,
};
use penumbra_stake::{
    validator::{self, ValidatorToml},
    IdentityKey,
};

use crate::App;

// TODO: replace this with something more standard for the `query` subcommand
#[derive(Debug, clap::Subcommand)]
pub enum ValidatorCmd {
    /// List all the validators in the network.
    List {
        /// Whether to show validators that are not currently part of the consensus set.
        #[clap(short = 'i', long)]
        show_inactive: bool,
        /// Whether to show detailed validator info.
        #[clap(short, long)]
        detailed: bool,
    },
    /// Fetch the current definition for a particular validator.
    Definition {
        /// The JSON file to write the definition to [default: stdout].
        #[clap(long)]
        file: Option<String>,
        /// The identity key of the validator to fetch.
        identity_key: String,
    },
}

impl ValidatorCmd {
    pub async fn exec(&self, app: &mut App) -> Result<()> {
        match self {
            ValidatorCmd::List {
                show_inactive,
                detailed,
            } => {
                let mut client = StakeQueryServiceClient::new(app.pd_channel().await?);

                let mut validators = client
                    .validator_info(ValidatorInfoRequest {
                        show_inactive: *show_inactive,
                        ..Default::default()
                    })
                    .await?
                    .into_inner()
                    .try_collect::<Vec<_>>()
                    .await?
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<validator::Info>, _>>()?;

                // Sort by voting power (descending), active first, then inactive
                validators.sort_by(|a, b| {
                    let av = if matches!(a.status.state, validator::State::Active) {
                        (a.status.voting_power, Amount::zero())
                    } else {
                        (Amount::zero(), a.status.voting_power)
                    };
                    let bv = if matches!(b.status.state, validator::State::Active) {
                        (b.status.voting_power, Amount::zero())
                    } else {
                        (Amount::zero(), b.status.voting_power)
                    };

                    bv.cmp(&av)
                });

                let total_voting_power = validators
                    .iter()
                    .filter_map(|v| {
                        if let validator::State::Active = v.status.state {
                            Some(v.status.voting_power.value())
                        } else {
                            None
                        }
                    })
                    .sum::<u128>() as f64;

                let mut table = Table::new();
                table.load_preset(presets::NOTHING);
                table.set_header(vec![
                    "Voting Power",
                    "Share",
                    "Commission",
                    "State",
                    "Bonding State",
                    "Validator Info",
                ]);

                for v in validators {
                    let voting_power = (v.status.voting_power.value() as f64) * 1e-6; // apply udelegation factor
                    let active_voting_power = if matches!(v.status.state, validator::State::Active)
                    {
                        v.status.voting_power.value() as f64
                    } else {
                        0.0
                    };
                    let power_percent = 100.0 * active_voting_power / total_voting_power;
                    let commission_bps = v
                        .validator
                        .funding_streams
                        .as_ref()
                        .iter()
                        .map(|fs| fs.rate_bps())
                        .sum::<u16>();

                    table.add_row(vec![
                        format!("{voting_power:.3}"),
                        format!("{power_percent:.2}%"),
                        format!("{commission_bps}bps"),
                        v.status.state.to_string(),
                        v.status.bonding_state.to_string(),
                        // TODO: consider rewriting this with term colors
                        // at some point, when we get around to it
                        v.validator.identity_key.to_string().red().to_string(),
                    ]);
                    table.add_row(vec![
                        "".into(),
                        "".into(),
                        "".into(),
                        "".into(),
                        "".into(),
                        v.validator.name.to_string().bright_green().to_string(),
                    ]);
                    if *detailed {
                        table.add_row(vec![
                            "".into(),
                            "".into(),
                            "".into(),
                            "".into(),
                            "".into(),
                            format!("  {}", v.validator.description),
                        ]);
                        table.add_row(vec![
                            "".into(),
                            "".into(),
                            "".into(),
                            "".into(),
                            "".into(),
                            format!("  {}", v.validator.website),
                        ]);
                    }
                }

                println!("{table}");
            }
            ValidatorCmd::Definition { file, identity_key } => {
                let identity_key = identity_key.parse::<IdentityKey>()?;

                /*
                use penumbra_proto::client::specific::ValidatorStatusRequest;

                let mut client = opt.specific_client().await?;
                let status: ValidatorStatus = client
                    .validator_status(ValidatorStatusRequest {
                        chain_id: "".to_string(), // TODO: fill in
                        identity_key: Some(identity_key.into()),
                    })
                    .await?
                    .into_inner()
                    .try_into()?;

                // why isn't the validator definition part of the status?
                // why do we have all these different validator messages?
                // do we need them?
                status.state.
                */

                // Intsead just download everything
                let mut client = StakeQueryServiceClient::new(app.pd_channel().await?);

                let validators = client
                    .validator_info(ValidatorInfoRequest {
                        show_inactive: true,
                        ..Default::default()
                    })
                    .await?
                    .into_inner()
                    .try_collect::<Vec<_>>()
                    .await?
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<Vec<validator::Info>, _>>()?;

                let validator: ValidatorToml = validators
                    .iter()
                    .map(|info| &info.validator)
                    .find(|v| v.identity_key == identity_key)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Could not find validator {}", identity_key))?
                    .into();

                if let Some(file) = file {
                    File::create(file)
                        .with_context(|| format!("cannot create file {file:?}"))?
                        .write_all(toml::to_string_pretty(&validator)?.as_bytes())
                        .context("could not write file")?;
                } else {
                    println!("{}", toml::to_string_pretty(&validator)?);
                }
            }
        }

        Ok(())
    }
}
