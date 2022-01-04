use once_cell::sync::Lazy;
use penumbra_crypto::asset;

mod delegate;
mod epoch;
mod funding_stream;
mod identity_key;
mod rate;
mod token;
mod undelegate;
mod validator;

pub use delegate::Delegate;
pub use epoch::Epoch;
pub use funding_stream::FundingStream;
pub use identity_key::IdentityKey;
pub use rate::{BaseRateData, RateData};
pub use token::DelegationToken;
pub use undelegate::Undelegate;
pub use validator::{Validator, ValidatorDefinition};

/// The Bech32 prefix used for validator consensus pubkeys.
pub const VALIDATOR_CONSENSUS_BECH32_PREFIX: &str = "penumbravalconspub";

pub use penumbra_proto::serializers::bech32str::validator_identity_key::BECH32_PREFIX as VALIDATOR_IDENTITY_BECH32_PREFIX;

// TODO: go through the source tree and use these instead of hardcoding "upenumbra"

pub static STAKING_TOKEN_DENOM: Lazy<asset::Denom> =
    Lazy::new(|| asset::REGISTRY.parse_denom("upenumbra").unwrap());
pub static STAKING_TOKEN_ASSET_ID: Lazy<asset::Id> =
    Lazy::new(|| asset::REGISTRY.parse_denom("upenumbra").unwrap().id());
