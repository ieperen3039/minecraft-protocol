use minecraft_protocol::data::block_states::BlockWithState;
use minecraft_protocol::data::blocks::Block;
use serde::{Deserialize, Serialize};
use std::ops::{Range, RangeInclusive};

/// Database for translating between block state ids and block ids.
// one block maps to multiple block states
// every block state maps to a single block
#[derive(Serialize, Deserialize)]
pub struct BlockRegistry {
    /// Maps a block id to the last block state id of that block.
    // we assume that no holes exist: if block `n` exists, then all blocks < `n` also exist.
    // we assume that if block `n` has state `m` then blocks > `n` have states > `m`
    // we use `u32` rather than BlockWithState because we will return Range<u32> from this list
    block_id_to_max_block_state_id: Vec<u32>,
    /// Maps a block state id to its block id
    block_state_id_to_block_id: Vec<Block>,
}

impl BlockRegistry {
    pub fn build(block_id_to_block_state_ids: Vec<(u32, RangeInclusive<u32>)>) -> BlockRegistry {
        let num_blocks = block_id_to_block_state_ids.len();
        let mut block_id_to_max_block_state_id = vec![0; num_blocks];

        let num_block_states = block_id_to_block_state_ids.iter().map(|(_, r)| *r.end()).max().unwrap() as usize + 1;
        let mut block_state_id_to_block_id = vec![Block::default(); num_block_states];

        for (block_id, state_ids) in block_id_to_block_state_ids {
            block_id_to_max_block_state_id[block_id as usize] = *state_ids.end();

            for state in state_ids {
                block_state_id_to_block_id[state as usize] = Block::from_id(block_id);
            }
        }

        BlockRegistry {
            block_id_to_max_block_state_id,
            block_state_id_to_block_id,
        }
    }

    pub fn block_is_valid(&self, block: Block) -> bool {
        (self.block_id_to_max_block_state_id.len() as u32) < block.id()
    }

    pub fn block_state_is_valid(&self, block_state: BlockWithState) -> bool {
        (self.block_state_id_to_block_id.len() as u32) < block_state.id()
    }

    /// returns the range of absolute state ids that map to this block.
    /// panics if the block is not valid
    pub fn block_to_block_states(&self, block: &Block) -> RangeInclusive<u32> {
        let num_blocks = self.block_id_to_max_block_state_id.len();
        let block_index = block.id() as usize;

        if block_index >= num_blocks {
            panic!(
                "Block id {} is higher than maximum of {}",
                block.id(),
                num_blocks
            )
        }

        let block_state_max = self.block_id_to_max_block_state_id[block_index];

        if block_index == 0 {
            return 0..=block_state_max
        }

        let block_state_min = self.block_id_to_max_block_state_id[block_index - 1] + 1;
        (block_state_min..=block_state_max).into()
    }

    /// returns the generalized block type of this block state
    /// panics if the block_state is not valid
    pub fn block_state_to_block(&self, block_state: &BlockWithState) -> Block {
        // panics if the block_state is not valid
        self.block_state_id_to_block_id[block_state.id() as usize]
    }
}
