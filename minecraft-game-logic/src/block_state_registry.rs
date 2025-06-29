use minecraft_protocol::data::block_states::BlockWithState;
use minecraft_protocol::data::blocks::Block;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::RangeInclusive;

/// Database for translating between block state ids and block ids.
// one block maps to one or more block states
// every block state maps to a single block
#[derive(Serialize, Deserialize)]
pub struct BlockStateRegistry {
    /// Maps a block id to the last block state id of that block.
    // we assume that no holes exist: if block `n` exists, then all blocks < `n` also exist.
    // we assume that if block `n` has state `m` then blocks > `n` have states > `m`
    // we use `u32` rather than BlockWithState because we will return Range<u32> from this list
    block_id_to_max_block_state_id: Vec<u32>,
    /// Maps a block to some default BlockWithState.
    block_id_to_default_state_id: Vec<BlockWithState>,
    /// Maps all block state ids to its respective block id
    block_state_id_to_block_id: Vec<Block>,
    /// A list of all states of blocks, to be indexed using block_state_lookup
    block_state_list: Vec<BlockState>,
    /// maps a block id to the last associated index in block_state_list
    block_state_lookup: Vec<usize>,
    /// total numer of all possible block states
    total_num_states: usize
}

/// Allows for the identification of a specific block state value, based on just the relative state
/// index of the block
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockState {
    /// the base block of this state
    pub block: Block,
    /// the multiplication of all num_values of all states of the base block before the target state
    /// or 1 if this is the first state
    pub offset: u32,
    /// the number of possible values in this state
    pub num_values: u32,
}

impl BlockStateRegistry {
    pub fn new() -> BlockStateRegistry {
        BlockStateRegistry{
            block_id_to_max_block_state_id : Vec::new(),
            block_id_to_default_state_id: Vec::new(),
            block_state_id_to_block_id : Vec::new(),
            block_state_list : Vec::new(),
            block_state_lookup : Vec::new(),
            total_num_states: 0,
        }
    }

    /// Adds the next block to this registry.
    /// Must be called in increasing order of block ids. Gaps are not allowed.
    /// Panics if the block id is not one higher than the previous one.
    pub fn add(&mut self, block: Block, state_sizes: Vec<u32>, default_state: BlockWithState) {
        let expected_block_id = self.block_state_list.last().map(|v| v.block.id() + 1).unwrap_or(0);
        assert_eq!(block.id(), expected_block_id);

        let mut multiplicative_offset = 1;
        for num_values in state_sizes {
            self.block_state_list.push(BlockState {
                block,
                offset: multiplicative_offset,
                num_values,
            });

            for _ in 0..num_values {
                self.block_state_id_to_block_id[self.total_num_states] = block;
                self.total_num_states += 1;
            }

            multiplicative_offset *= num_values;
        }

        self.block_state_lookup.push(self.block_state_list.len());
        self.block_id_to_max_block_state_id.push((self.total_num_states - 1) as u32);
        self.block_id_to_default_state_id.push(default_state);
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

    /// Overwrites the numerical value of the state represented by target_state.
    /// If target_state is not based off the given block_state, the result is undefined.
    pub fn set_block_state_value(
        &self,
        block_state: BlockWithState,
        target_state: BlockState,
        new_value: u32,
    ) -> BlockWithState {
        let block = target_state.block;
        debug_assert_eq!(block, self.block_state_to_block(block_state));
        assert!(new_value < target_state.num_values);

        let states = self.block_to_block_states(block);
        let mut relative_block_state = block_state.id() - states.start();

        // there might be a faster method, but this is not too expensive
        let current_value = (relative_block_state / target_state.offset) % target_state.num_values;
        relative_block_state -= current_value * target_state.offset;
        relative_block_state += new_value * target_state.offset;

        BlockWithState::from_id(relative_block_state + states.start())
    }
}
