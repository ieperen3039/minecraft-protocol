use minecraft_protocol::data::block_states::BlockWithState;
use minecraft_protocol::data::blocks::Block;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

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
    /// A list of all states of blocks, to be indexed using block_state_lookup
    block_state_list: Vec<BlockState>,
    /// maps a block id to the last associated index in block_state_list
    block_state_lookup: Vec<usize>,
}

/// Allows for the identification of a specific block state value, based on just the relative state
/// index of the block
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockState {
    /// the base block of this state
    pub block: Block,
    /// the multiplication of all num_values of all states of the base block before the target state
    pub offset: u32,
    /// the number of possible values in this state
    pub num_values: u32,
}

impl BlockRegistry {
    pub fn build(block_states: Vec<BlockState>, num_blocks: usize) -> BlockRegistry {
        let mut block_id_to_max_block_state_id = vec![0; num_blocks];
        let mut block_id_to_num_block_state_values = vec![0; num_blocks];
        let mut block_state_lookup = vec![0; num_blocks];

        for (block_state_index, state) in block_states.iter().enumerate() {
            let block_index = state.block.id() as usize;

            // make sure block_state_lookup contains the highest state index
            let lookup_value = &mut block_state_lookup[block_index];
            *lookup_value = usize::max(*lookup_value, block_state_index);

            // count the number of block states for this block
            block_id_to_num_block_state_values[block_index] += state.num_values as usize;
        }

        let num_block_state_values = block_id_to_num_block_state_values.iter().sum();
        let mut block_state_id_to_block_id = vec![Block::default(); num_block_state_values];

        // now we have collected the total number of states for each block, we can calculate the max state value for each block
        let mut num_block_state_values_counter = 0;
        for (block_index, num_values) in block_id_to_num_block_state_values.iter().enumerate() {
            num_block_state_values_counter += num_values;

            block_id_to_max_block_state_id[block_index as u32] = num_block_state_values_counter - 1;
        }

        BlockRegistry {
            block_id_to_max_block_state_id,
            block_state_id_to_block_id,
            block_state_list : block_states,
            block_state_lookup
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
    pub fn block_to_block_states(&self, block: Block) -> RangeInclusive<u32> {
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
            return 0..=block_state_max;
        }

        let block_state_min = self.block_id_to_max_block_state_id[block_index - 1] + 1;
        (block_state_min..=block_state_max).into()
    }

    /// returns the generalized block type of this block state
    /// panics if the block_state is not valid
    pub fn block_state_to_block(&self, block_state: BlockWithState) -> Block {
        // panics if the block_state is not valid
        self.block_state_id_to_block_id[block_state.id() as usize]
    }

    /// Returns the numerical value of the state represented by target_state.
    /// If target_state is not based off the given block_state, the result is undefined
    pub fn get_block_state_value(
        &self,
        block_state: BlockWithState,
        target_state: BlockState,
    ) -> u32 {
        let block = target_state.block;
        debug_assert_eq!(block, self.block_state_to_block(block_state));

        let states = self.block_to_block_states(block);
        let relative_block_state = block_state.id() - states.start();

        (relative_block_state / target_state.offset) % target_state.num_values
    }
}
