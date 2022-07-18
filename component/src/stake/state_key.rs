use jmt::KeyHash;
use penumbra_crypto::IdentityKey;
use tendermint::{block, PublicKey};

pub fn validators(id: &IdentityKey) -> KeyHash {
    format!("staking/validators/{}", id).into()
}

pub fn state_by_validator(id: &IdentityKey) -> KeyHash {
    format!("staking/validators/{}/state", id).into()
}

pub fn current_rate_by_validator(id: &IdentityKey) -> KeyHash {
    format!("staking/validators/{}/rate/current", id).into()
}

pub fn next_rate_by_validator(id: &IdentityKey) -> KeyHash {
    format!("staking/validators/{}/rate/next", id).into()
}

pub fn power_by_validator(id: &IdentityKey) -> KeyHash {
    format!("staking/validators/{}/power", id).into()
}

pub fn bonding_state_by_validator(id: &IdentityKey) -> KeyHash {
    format!("staking/validators/{}/bonding_state", id).into()
}

pub fn uptime_by_validator(id: &IdentityKey) -> KeyHash {
    format!("staking/validator_uptime/{}", id).into()
}

pub fn slashed_validators(height: u64) -> KeyHash {
    format!("staking/slashed_validators/{}", height).into()
}

pub fn consensus_key(pk: &PublicKey) -> KeyHash {
    format!("staking/consensus_key/{}", pk.to_hex()).into()
}

pub fn delegation_changes_by_height(height: u64) -> KeyHash {
    format!("staking/delegation_changes/{}", height).into()
}
