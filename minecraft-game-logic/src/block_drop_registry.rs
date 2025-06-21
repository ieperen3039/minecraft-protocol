use crate::block_state_registry::BlockRegistry;
use minecraft_protocol::data::{block_states::BlockWithState, blocks::Block, items::Item};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Database for translating blocks to item drops.
/// Use `get_drops` to query.
// Some
#[derive(Serialize, Deserialize)]
pub struct BlockDropRegistry {
    // Which tools will result in a drop from the `tool_drops` table.
    // NOTE: for some blocks, this is different from the "appropriate" tool
    block_tools: HashMap<Block, Vec<Item>>,
    // What drops when mined using the appropriate tool.
    // If the block is not in this list, hand_drops is used
    tool_drops: HashMap<BlockWithState, DropTable>,
    // What drops when mined with silk touch
    // If the block is not in this list, hand_drops is used
    silk_touch_drops: HashMap<BlockWithState, DropTable>,
    // What drops when mined by hand.
    // If the block is not in this list, nothing is dropped
    hand_drops: HashMap<BlockWithState, DropTable>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DropTable {
    // single kind of item
    Single(ItemDrop),
    // multiple different items
    MultipleIndependent(Vec<ItemDrop>),
    // one of multiple possible drops, like gravel
    OneOfMultiple(Vec<WeightedDrop>),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ItemDrop {
    pub item: Item,
    pub quantity: ItemDropQuantity,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum ItemDropQuantity {
    /** Exactly one of the item, like stone */
    Single,
    /** Always a fixed quantity of the item, like 4-sided vines */
    FixedMultiple { quantity: usize },
    /** A random quantity unaffected by fortune */
    RandomRange {
        min: usize,
        max: usize, // inclusive
    },
    /** A random quantity. Fortune increases the maximum number of drops. */
    RandomRangeFortune {
        min: usize,
        max: usize, // inclusive
        fortune_increase: usize,
    },
    /**
     * A random quantity with a strict upper limit. Fortune increases the maximum number of drops.
     * If a drop higher than the maximum is rolled, it is rounded down to the capacity.
     */
    RandomRangeFortuneMax {
        min: usize,
        max: usize, // inclusive
        fortune_increase: usize,
        capacity: usize,
    },
    /**
     * A random quantity multiplied by fortune.
     * Fortune gives a weight of 2 to a normal drop chance and adds a weight of 1 for each extra drop multiplier.
     * Every level of fortune adds a multiplier of `level+1` to the table.
     * If a bonus is applied, there is an equal chance for any number of drops between 2 and `level+1`.
     */
    RandomRangeMultiplier {
        min: usize,
        max: usize, // inclusive
    },
    /** one or zero, based on chance, unaffected by fortune */
    RandomChance { chance: f32 },
    /** one or zero, based on chance, with fortune effect from a table */
    ChanceFromTable {
        // Every index refers to the level of fortune
        chance: [f32; 5],
    },
    /**
     * Zero or more, affected by fortune.
     * Seeds use a binomial distribution by rolling `min` number of times with a given drop probability.
     * Fortune increases the number of tests for the distribution, and thus the maximum number of drops.
     */
    RandomChanceSeeds {
        chance: f32,
        num_rolls: usize,
        fortune_increase: usize,
    },
    /**
     * Zero or more, affected by fortune.
     * Short grass and ferns have a fixed chance, unaffected by Fortune, to drop wheat seeds.
     * If the drop occurs, Fortune increases the maximum number of seeds that can be dropped.
     */
    RandomChanceGrass {
        chance: f32,
        min: usize,
        max: usize, // inclusive
        fortune_increase: usize,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WeightedDrop {
    // Every index refers to the level of fortune
    weight: [u32; 5],
    result: ItemDrop,
}

impl DropTable {
    pub fn from_vec(mut drops: Vec<ItemDrop>) -> Option<DropTable> {
        match drops.len() {
            0 => None,
            1 => Some(DropTable::Single(drops.pop().unwrap())),
            _ => Some(DropTable::MultipleIndependent(drops)),
        }
    }
}

impl BlockDropRegistry {
    pub fn new() -> Self {
        Self {
            block_tools: HashMap::new(),
            tool_drops: HashMap::new(),
            silk_touch_drops: HashMap::new(),
            hand_drops: HashMap::new(),
        }
    }

    /// Overrides all existing drops for the given block.
    pub fn set_block_drops(
        &mut self,
        block: BlockWithState,
        with_tool: Option<DropTable>,
        with_silk_touch: Option<DropTable>,
        with_hands: Option<DropTable>,
    ) {
        if let Some(with_tool) = with_tool {
            self.tool_drops.insert(block, with_tool);
        }

        if let Some(with_hands) = with_hands {
            self.hand_drops.insert(block, with_hands);
        }

        if let Some(with_silk_touch) = with_silk_touch {
            self.silk_touch_drops.insert(block, with_silk_touch);
        }
    }

    /// Appends the given list of tools to the set of accepted tools for the given block
    pub fn set_tools(&mut self, block: Block, tools: Vec<Item>) {
        self.block_tools.insert(block, tools);
    }

    /// Fills the drops_out vector with the drops for the given block.
    /// Multiple calls with the same arguments will yield different results, due to rng.
    pub fn get_drops(
        &self,
        block_registry: BlockRegistry,
        block_state: BlockWithState,
        held_item: Item,
        silk_touch: bool,
        fortune: u32,
        rng: &mut dyn RngCore,
        drops_out: &mut Vec<Item>,
    ) {
        if silk_touch {
            if let Some(drop) = self.silk_touch_drops.get(&block_state) {
                Self::process_drop_table(drop, fortune, rng, drops_out);
            }
            // else the block already drops itself, or never drops anything
        }

        let block = block_registry.block_state_to_block(&block_state);

        if let Some(tools) = self.block_tools.get(&block) {
            if tools.contains(&held_item) {
                if let Some(tool_drop) = self.tool_drops.get(&block_state) {
                    Self::process_drop_table(tool_drop, fortune, rng, drops_out);
                    return;
                }
                // else it drops the same as when mined by hand
            }
            // else the inappropriate tool is used, and we use hand_drops.
        }

        if let Some(hand_drop) = self.hand_drops.get(&block_state) {
            Self::process_drop_table(hand_drop, fortune, rng, drops_out);
            return;
        }

        // else drop nothing
    }

    fn process_drop_table(
        drop_logic: &DropTable,
        fortune: u32,
        rng: &mut dyn RngCore,
        drops_out: &mut Vec<Item>,
    ) {
        match drop_logic {
            DropTable::Single(drop) => Self::process_item_drop(drop, fortune, rng, drops_out),
            DropTable::MultipleIndependent(drops) => {
                for drop in drops {
                    Self::process_item_drop(drop, fortune, rng, drops_out);
                }
            }
            DropTable::OneOfMultiple(drops) => {
                let total_of_weights = drops.iter().map(|d| d.weight[fortune as usize]).sum();

                let mut target_weight = get_random_int(rng, total_of_weights);
                for drop in drops {
                    let weight = drop.weight[fortune as usize];
                    if target_weight >= weight {
                        target_weight -= weight;
                    } else {
                        Self::process_item_drop(&drop.result, fortune, rng, drops_out);
                    }
                }
            }
        }
    }

    fn process_item_drop(
        drop_logic: &ItemDrop,
        fortune: u32,
        rng: &mut dyn RngCore,
        drops_out: &mut Vec<Item>,
    ) {
        let quantity = match &drop_logic.quantity {
            ItemDropQuantity::Single => 1,
            ItemDropQuantity::FixedMultiple{ quantity } => *quantity,
            ItemDropQuantity::RandomRange { min, max } => {
                min + get_random_int(rng, (max - min) as u32) as usize
            }
            ItemDropQuantity::RandomChance { chance } => {
                if try_random_chance(rng, *chance) {
                    1
                } else {
                    0
                }
            }
            ItemDropQuantity::ChanceFromTable { chance } => {
                if try_random_chance(rng, chance[fortune as usize]) {
                    1
                } else {
                    0
                }
            }
            ItemDropQuantity::RandomRangeFortune {
                min,
                max,
                fortune_increase,
            } => {
                let max = max + (fortune as usize) * fortune_increase;
                min + get_random_int(rng, (max - min) as u32) as usize
            }
            ItemDropQuantity::RandomRangeFortuneMax {
                min,
                max,
                capacity,
                fortune_increase,
            } => {
                let max = max + (fortune as usize) * fortune_increase;
                let drops = min + get_random_int(rng, (max - min) as u32) as usize;
                usize::min(drops, usize::from(*capacity))
            }
            ItemDropQuantity::RandomRangeMultiplier { min, max } => {
                let total_of_weights = 2 + fortune;
                let target_weight = get_random_int(rng, total_of_weights);
                let multiplier = u32::max(target_weight - 2, 1);

                let drops = min + get_random_int(rng, (max - min) as u32) as usize;

                drops * multiplier as usize
            }
            ItemDropQuantity::RandomChanceSeeds {
                chance,
                num_rolls,
                fortune_increase,
            } => {
                // TODO we could sample the rng once, and use a pre-calculated weighted lookup-table
                let mut num_drops = 0;
                for _ in 0..num_rolls + fortune_increase {
                    if try_random_chance(rng, *chance) {
                        num_drops += 1;
                    }
                }
                num_drops
            }
            ItemDropQuantity::RandomChanceGrass {
                chance,
                min,
                max,
                fortune_increase,
            } => {
                if try_random_chance(rng, *chance) {
                    let max = max + fortune_increase * fortune as usize;
                    min + get_random_int(rng, (max - min) as u32) as usize
                } else {
                    0
                }
            }
        };

        for _ in 0..quantity {
            drops_out.push(drop_logic.item.clone());
        }
    }
}

// Returns a float between 0.0 and 1.0, exclusive.
// May return 0.0, will never return 1.0.
fn get_random_float(rng: &mut dyn RngCore) -> f32 {
    // We add one to the denominator, to avoid getting exactly 1.0.
    (rng.next_u32() as f32) / (u32::MAX as f32 + 1.0)
}

fn get_random_int(rng: &mut dyn RngCore, highest_value: u32) -> u32 {
    // convert a random u32 to a float, this gives less biased random results
    let random_float = get_random_float(rng);
    // the following logic gives a fair distribution from 0 to highest_value, inclusive
    let fractional_add = (highest_value + 1) as f32 * random_float;
    fractional_add.floor() as u32
}

fn try_random_chance(rng: &mut dyn RngCore, chance: f32) -> bool {
    // if chance == 1.0, this will always return true
    // if chance == 0.0, this will always return false
    get_random_float(rng) < chance
}
