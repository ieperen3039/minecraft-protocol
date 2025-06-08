use minecraft_protocol::data::{blocks::Block, items::Item};
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct BlockDrops {
    // Which tools will result in a drop from the `tool_drops` table
    block_tools: HashMap<Block, Vec<Item>>,
    // What drops when mined using the appropriate tool
    tool_drops: HashMap<Block, DropTable>,
    // What drops when mined by hand
    hand_drops: HashMap<Block, DropTable>,
    // What drops when mined with silk touch
    silk_touch_drops: HashMap<Block, DropTable>,
}

#[derive(Serialize, Deserialize)]
pub enum DropTable {
    // single kind of item
    Single(ItemDrop),
    // multiple different items
    MultipleIndependent(Vec<ItemDrop>),
    // one of multiple possible drops, like gravel
    OneOfMultiple(Vec<WeightedDrop>),
}

#[derive(Serialize, Deserialize)]
pub struct ItemDrop {
    pub item: Item,
    pub quantity: ItemDropQuantity,
}

#[derive(Serialize, Deserialize)]
pub enum ItemDropQuantity {
    /** Exactly one of the item, like stone */
    Single,
    /** Always a fixed quantity of the item, like 4-sided vines */
    FixedMultiple(usize),
    /** A random quantity unaffected by fortune */
    RandomRange {
        min: usize,
        max: usize, // inclusive
    },
    /** A random quantity. Fortune increases the maximum number of drops by 1 per level. */
    RandomRangeFortune {
        min: usize,
        max: usize, // inclusive
    },
    /**
     * A random quantity with a strict upper limit. Fortune increases the maximum number of drops by 1 per level.
     * If a drop higher than the maximum is rolled, it is rounded down to the capacity.
     */
    RandomRangeFortuneMax {
        min: usize,
        max: usize, // inclusive
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

#[derive(Serialize, Deserialize)]
pub struct WeightedDrop {
    // Every index refers to the level of fortune
    weight: [u32; 5],
    result: ItemDrop,
}

impl BlockDrops {
    pub fn new() -> Self {
        Self {
            block_tools: HashMap::new(),
            tool_drops: HashMap::new(),
            hand_drops: HashMap::new(),
            silk_touch_drops: HashMap::new(),
        }
    }

    pub fn get_drops(
        &self,
        block: Block,
        held_item: Item,
        silk_touch: bool,
        fortune: u32,
        rng: &mut dyn RngCore,
        drops_out: &mut Vec<Item>,
    ) {
        if silk_touch {
            if let Some(drop) = self.silk_touch_drops.get(&block) {
                Self::process_drop_table(drop, fortune, rng, drops_out);
            }
            // else it drops nothing
            return;
        }

        if let Some(tools) = self.block_tools.get(&block) {
            if tools.contains(&held_item) {
                if let Some(tool_drop) = self.tool_drops.get(&block) {
                    Self::process_drop_table(tool_drop, fortune, rng, drops_out);
                    return;
                }
                // else it drops nothing
                return;
            }
            // else the inappropriate tool is used, and we use hand_drops.
        }

        if let Some(hand_drop) = self.hand_drops.get(&block) {
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
            ItemDropQuantity::FixedMultiple(quantity) => *quantity,
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
            ItemDropQuantity::RandomRangeFortune { min, max } => {
                let max = max + fortune as usize;
                min + get_random_int(rng, (max - min) as u32) as usize
            }
            ItemDropQuantity::RandomRangeFortuneMax { min, max, capacity } => {
                let max = max + fortune as usize;
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
