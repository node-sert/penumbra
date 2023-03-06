use anyhow::Result;
use async_trait::async_trait;
use penumbra_crypto::dex::{
    lp::{
        position::{self, Position},
        LpNft, Reserves,
    },
    DirectedTradingPair,
};
use penumbra_proto::{DomainType, StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};

use super::state_key;

#[async_trait]
pub trait PositionRead: StateRead {
    async fn position_by_id(&self, id: &position::Id) -> Result<Option<position::Metadata>> {
        self.get(&state_key::position_by_id(id)).await
    }

    async fn check_position_id_unused(&self, id: &position::Id) -> Result<()> {
        match self.get_raw(&state_key::position_by_id(id)).await? {
            Some(_) => Err(anyhow::anyhow!("position id {:?} already used", id)),
            None => Ok(()),
        }
    }
}
impl<T: StateRead + ?Sized> PositionRead for T {}

/// Manages liquidity positions within the chain state.
#[async_trait]
pub trait PositionManager: StateWrite + PositionRead {
    /// Validates position arguments and records the new position in the chain state.
    async fn position_open(
        &mut self,
        position: Position,
        initial_reserves: Reserves,
    ) -> Result<()> {
        let id = position.id();

        let metadata = position::Metadata {
            position,
            state: position::State::Opened,
            reserves: initial_reserves,
        };
        self.index_position(&metadata);
        self.put_position(&id, metadata);

        Ok(())
    }

    /// Marks an existing position as closed in the chain state.
    async fn position_close(&mut self, id: &position::Id) -> Result<()> {
        let mut metadata = self
            .position_by_id(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("could not find position with id {}", id))?;
        if metadata.state != position::State::Opened {
            return Err(anyhow::anyhow!(
                "attempted to close position {} with state {}",
                id,
                metadata.state
            ));
        }

        metadata.state = position::State::Closed;
        self.deindex_position(&metadata.position);
        self.put_position(id, metadata);

        Ok(())
    }

    /// Marks an existing closed position as withdrawn in the chain state.
    async fn position_withdraw(&mut self, _id: &position::Id) {
        todo!()
    }

    /// Marks an existing withdrawn position as claimed in the chain state.
    async fn position_reward_claim(&mut self, _id: &position::Id) {
        todo!()
    }
}

impl<T: StateWrite + ?Sized> PositionManager for T {}

#[async_trait]
trait Inner: StateWrite {
    fn put_position(&mut self, id: &position::Id, metadata: position::Metadata) {
        self.put(state_key::position_by_id(id), metadata);
    }

    fn index_position(&mut self, metadata: &position::Metadata) {
        let (pair, phi) = (metadata.position.phi.pair, &metadata.position.phi);
        let id_bytes = metadata.position.id().encode_to_vec();
        if metadata.reserves.r1 != 0u64.into() {
            // Index this position for trades FROM asset 2 TO asset 1, since the position has asset 1 to give out.
            let pair = DirectedTradingPair {
                start: pair.asset_1(),
                end: pair.asset_2(),
            };
            let phi = phi.component.clone();
            self.nonconsensus_put_raw(
                state_key::internal::price_index::key(&pair, &phi),
                id_bytes.clone(),
            );
        }
        if metadata.reserves.r2 != 0u64.into() {
            // Index this position for trades FROM asset 1 TO asset 2, since the position has asset 2 to give out.
            let pair = DirectedTradingPair {
                start: pair.asset_2(),
                end: pair.asset_1(),
            };
            let phi = phi.component.flip();
            self.nonconsensus_put_raw(state_key::internal::price_index::key(&pair, &phi), id_bytes);
        }
    }

    fn deindex_position(&mut self, position: &Position) {
        let pair12 = DirectedTradingPair {
            start: position.phi.pair.asset_1(),
            end: position.phi.pair.asset_2(),
        };
        let phi12 = position.phi.component.clone();
        let pair21 = DirectedTradingPair {
            start: position.phi.pair.asset_2(),
            end: position.phi.pair.asset_1(),
        };
        let phi21 = position.phi.component.flip();
        self.nonconsensus_delete(state_key::internal::price_index::key(&pair12, &phi12));
        self.nonconsensus_delete(state_key::internal::price_index::key(&pair21, &phi21));
    }
}
impl<T: StateWrite + ?Sized> Inner for T {}
