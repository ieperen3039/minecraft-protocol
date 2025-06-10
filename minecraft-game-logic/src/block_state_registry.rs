use std::ops::Range;
use minecraft_protocol::data::block_states::BlockWithState;
use minecraft_protocol::data::blocks::Block;
use serde::{Deserialize, Serialize};

/// Database for translating between block state ids and block ids.
// one block maps to multiple block states
// every block state maps to a single block
#[derive(Serialize, Deserialize)]
pub struct BlockRegistry {
    /// Maps a block id to the first block state id of that block.
    // we assume that no holes exist: if block `n` exists, then all blocks < `n` also exist.
    // we assume that if block `n` has state `m` then blocks > `n` have states > `m`
    // hence, as every index refers to a block id, the array is sorted on state id.
    block_id_to_block_state_id: Vec<BlockWithState>,
}

impl BlockRegistry {
    pub fn is_valid(&self, block: Block) -> bool {
        // true if such an element exists in block_id_to_block_state_id
        (self.block_id_to_block_state_id.len() as u32) < block.id()
    }

    pub fn block_to_block_states(&self, block: &Block) -> Range<BlockWithState> {
        // assuming that the first block state is the default
        let block_id_index = block.id() as usize;
        let block_state = self.block_id_to_block_state_id[block_id_index];
        let block_state_max = self.block_id_to_block_state_id[block_id_index + 1];
        block_state .. block_state_max
    }

    pub fn block_state_to_block(&self, block_state: &BlockWithState) -> Block {
        let index = self
            .block_id_to_block_state_id
            .binary_search(block_state)
            .expect(format!("no block maps to state id {}", block_state.id()).as_str());

        // if index wasn't valid, it would not be returned as Ok from binary_search
        Block::from_id(index as u32)
    }
}
