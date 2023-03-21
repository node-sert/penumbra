use crate::sct::state_key;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use penumbra_chain::StateReadExt as _;
use penumbra_proto::{StateReadProto, StateWriteProto};
use penumbra_storage::{StateRead, StateWrite};
use penumbra_tct as tct;

#[async_trait]
pub(crate) trait StateWriteExt: StateWrite {
    fn stub_put_state_commitment_tree(&mut self, tree: &tct::Tree) {
        let bytes = bincode::serialize(&tree).unwrap();
        self.nonconsensus_put_raw(
            state_key::stub_state_commitment_tree().as_bytes().to_vec(),
            bytes,
        );
    }

    fn set_sct_anchor(&mut self, height: u64, sct_anchor: tct::Root) {
        tracing::debug!(?height, ?sct_anchor, "writing anchor");

        self.put(state_key::anchor_by_height(height), sct_anchor);
        self.put_proto(state_key::anchor_lookup(sct_anchor), height);
    }

    fn set_sct_block_anchor(&mut self, height: u64, sct_block_anchor: tct::builder::block::Root) {
        tracing::debug!(?height, ?sct_block_anchor, "writing block anchor");

        self.put(state_key::block_anchor_by_height(height), sct_block_anchor);
        self.put_proto(state_key::block_anchor_lookup(sct_block_anchor), height);
    }

    fn set_sct_epoch_anchor(&mut self, index: u64, sct_block_anchor: tct::builder::epoch::Root) {
        tracing::debug!(?index, ?sct_block_anchor, "writing epoch anchor");

        self.put(state_key::epoch_anchor_by_index(index), sct_block_anchor);
        self.put_proto(state_key::epoch_anchor_lookup(sct_block_anchor), index);
    }

    async fn write_sct(
        &mut self,
        height: u64,
        sct: tct::Tree,
        block_root: tct::builder::block::Root,
        epoch_root: Option<tct::builder::epoch::Root>,
    ) {
        self.set_sct_anchor(height, sct.root());

        self.set_sct_block_anchor(height, block_root);

        if let Some(epoch_root) = epoch_root {
            let index = self.epoch().await.expect("epoch must be set").index;
            self.set_sct_epoch_anchor(index, epoch_root);
        }

        self.stub_put_state_commitment_tree(&sct);
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}

#[async_trait]
pub trait StateReadExt: StateRead {
    async fn stub_state_commitment_tree(&self) -> tct::Tree {
        match self
            .nonconsensus_get_raw(state_key::stub_state_commitment_tree().as_bytes())
            .await
            .unwrap()
        {
            Some(bytes) => bincode::deserialize(&bytes).unwrap(),
            None => tct::Tree::new(),
        }
    }

    async fn anchor_by_height(&self, height: u64) -> Result<Option<tct::Root>> {
        self.get(&state_key::anchor_by_height(height)).await
    }

    async fn check_claimed_anchor(&self, anchor: tct::Root) -> Result<()> {
        if anchor.is_empty() {
            return Ok(());
        }

        if let Some(anchor_height) = self
            .get_proto::<u64>(&state_key::anchor_lookup(anchor))
            .await?
        {
            tracing::debug!(?anchor, ?anchor_height, "anchor is valid");
            Ok(())
        } else {
            Err(anyhow!(
                "provided anchor {} is not a valid SCT root",
                anchor
            ))
        }
    }
}

impl<T: StateRead + ?Sized> StateReadExt for T {}
