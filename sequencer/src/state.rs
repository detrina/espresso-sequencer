use crate::{Block, ChainVariables, Error, SequencerTransaction};
use commit::{Commitment, Committable};
use hotshot::traits::State as HotShotState;
use hotshot_types::{data::ViewNumber, traits::state::ConsensusTime};
#[allow(deprecated)]
use nll::nll_todo::nll_todo;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, ops::Deref};

#[derive(Serialize, Deserialize, Clone, Debug, Hash, PartialEq, Eq)]
pub struct State {
    chain_variables: ChainVariables,
    block_height: u64,
    view_number: ViewNumber,
    prev_state_commitment: Option<Commitment<Self>>,
}

impl Default for State {
    fn default() -> Self {
        State {
            chain_variables: Default::default(),
            block_height: Default::default(),
            view_number: ViewNumber::genesis(),
            prev_state_commitment: Default::default(),
        }
    }
}

impl HotShotState for State {
    type Error = Error;

    type BlockType = Block;

    type Time = ViewNumber;

    fn next_block(&self) -> Self::BlockType {
        #[allow(deprecated)]
        nll_todo()
    }

    fn validate_block(&self, block: &Self::BlockType, view_number: &Self::Time) -> bool {
        // Parent state commitment of block must match current state commitment
        if block.parent_state != self.commit() {
            return false;
        }

        // New view number must be greater than current
        if *view_number <= self.view_number {
            return false;
        }

        if self.block_height == 0 {
            // Must contain only a genesis transaction
            if block.transactions.len() != 1
                || !matches!(block.transactions[0], SequencerTransaction::Genesis(_))
            {
                return false;
            }
        } else {
            // If any txn is Genesis, fail
            if block
                .transactions
                .iter()
                .any(|txn| matches!(txn, SequencerTransaction::Genesis(_)))
            {
                return false;
            }
        }
        true
    }

    fn append(
        &self,
        block: &Self::BlockType,
        view_number: &Self::Time,
    ) -> Result<Self, Self::Error> {
        // Have to save state commitment here if any changes are made

        if !self.validate_block(block, view_number) {
            return Err(Error::ValidationError);
        }

        Ok(State {
            chain_variables: self.chain_variables.clone(),
            block_height: self.block_height + 1,
            view_number: *view_number,
            prev_state_commitment: Some(self.commit()),
        })
    }

    fn on_commit(&self) {}
}

impl Committable for State {
    fn commit(&self) -> Commitment<Self> {
        commit::RawCommitmentBuilder::new("State")
            .field("chain_variables", self.chain_variables.commit())
            .u64_field("block_height", self.block_height)
            .u64_field("view_number", *self.view_number.deref())
            .array_field(
                "prev_state_commitment",
                &self
                    .prev_state_commitment
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>(),
            )
            .finalize()
    }
}
