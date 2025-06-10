use minecraft_external::json;
use minecraft_game_logic::block_drop_registry::*;
use minecraft_game_logic::block_state_registry::BlockRegistry;
use minecraft_protocol::data::block_states::BlockWithState;
use minecraft_protocol::data::{blocks::Block, items::Item};
use std::fs::File;
use std::io::Write;

// Currently impossible to encode:
//  * sea pickles have "stackSizeRange": [4, 4] and state data with the name "pickles" that specify how much must be dropped
//  * every type of candle has "stackSizeRange": [4, 4] and state data with the name "candles" that specify how much must be dropped
//  * beehive drop bee entities (if there are bees in the hive, which is not part of the state data)

pub fn generate_block_drop_binary(
    block_registry: BlockRegistry,
    block_drops: &Vec<json::BlockDropMapping>,
    blocks: &Vec<json::Block>,
    items: &Vec<json::Item>,
    file: &mut File,
) {
    let mut registry = BlockDropRegistry::new();

    for drop_mapping in block_drops {
        let block = blocks
            .iter()
            .find(|b| b.internal_name == drop_mapping.block_internal_name)
            .expect(format!("Block {} not found", drop_mapping.block_internal_name).as_str());

        let tools: Vec<Item> = block
            .harvest_tools
            .iter()
            .filter_map(|(tool, is_tool)| if *is_tool { Some(*tool) } else { None })
            .map(Item::from_id)
            .collect();

        registry.set_tools(Block::from_id(block.id), tools);

        let mut with_silk_touch: Vec<ItemDrop> = Vec::new();
        let mut with_tool: Vec<ItemDrop> = Vec::new();
        let mut with_hands: Vec<ItemDrop> = Vec::new();

        for drop_entry in drop_mapping.drops {
            let item = items
                .iter()
                .find(|item| item.internal_name == drop_entry.item_internal_name)
                .expect(format!("Item {} not found", drop_entry.item_internal_name).as_str())
                .to_owned();

            let item = Item::from_id(item.id);

            // TODO drop_entry.drop_chance is broken
            let quantity = {
                // TODO this must be far more advanced
                let [min, max] = drop_entry.stack_size_range;
                if drop_entry.no_silk_touch == Some(true)
                    && drop_mapping.block_internal_name.ends_with("_ore")
                {
                    // special case: ore
                    ItemDropQuantity::RandomRangeMultiplier {
                        min: min.unwrap_or(1) as usize,
                        max: max.unwrap_or(1) as usize,
                    }
                } else if min == Some(-1) {
                    ItemDropQuantity::Single
                } else if min == max {
                    // also when both are None
                    ItemDropQuantity::Single
                } else if min.is_some() && max.is_some() {
                    ItemDropQuantity::RandomRange {
                        min: min.unwrap() as usize,
                        max: max.unwrap() as usize,
                    }
                } else {
                    panic!("min == {} && max == {}", min, max)
                }
            };

            if drop_entry.silk_touch == Some(true) {
                with_silk_touch.push(ItemDrop { item, quantity });
            }

            if drop_entry.no_silk_touch == Some(true) {
                with_tool.push(ItemDrop { item, quantity });
            }

            if drop_entry.silk_touch.is_none() && drop_entry.no_silk_touch.is_none() {
                with_hands.push(ItemDrop { item, quantity });
            }
        }

        let with_tool = to_drop_table(with_tool);
        let with_hands = to_drop_table(with_hands);
        let with_silk_touch = to_drop_table(with_silk_touch);

        let states = block_registry.block_to_block_states(&Block::from_id(block.id));

        for state in states {
            registry.set_block_drops(
                state,
                with_tool.clone(),
                with_hands.clone(),
                with_silk_touch.clone(),
            );
        }
    }

    /// handle special cases
    handle_leaves(blocks, items, &mut registry);
    handle_candles(blocks, items, &mut registry);
    handle_sea_pickles(blocks, items, &mut registry);


    todo!();

    let encoded = bincode::serde::encode_to_vec(&registry, bincode::config::standard())
        .expect("Failed to encode recipes");

    file.write_all(&encoded).unwrap()
}

fn handle_candles(blocks: &Vec<json::Block>, items: &Vec<json::Item>, registry: &mut BlockDropRegistry) {
    // every type of candle has state data with the name "candles" that specify how much must be dropped
    let candles = blocks
        .iter()
        .filter(|b| b.states.iter().any(|s| s.name == "candles"));

    for candle_block in candles {
        if let Some(candle_item) = items
            .iter()
            .find(|b| b.internal_name == candle_block.internal_name)
        {
            for state in candle_block.min_state_id..=candle_block.max_state_id {
                // assuming num candles is the least significant state number
                let num_drops = (state - candle_block.min_state_id) as usize % 4;
                registry.set_block_drops(
                    BlockWithState::from_id(state),
                    None,
                    None,
                    Some(DropTable::Single(ItemDrop {
                        item: Item::from_id(candle_item.id),
                        quantity: ItemDropQuantity::FixedMultiple(num_drops),
                    })),
                )
            }
        }
    }
}

fn handle_sea_pickles(blocks: &Vec<json::Block>, items: &Vec<json::Item>, registry: &mut BlockDropRegistry) {
    // sea pickles have state data with the name "pickles" that specify how much must be dropped
    if let (Some(sea_pickle_block), Some(sea_pickle_item)) = (
        blocks
            .iter()
            .find(|b| b.internal_name == "minecraft:sea_pickle"),
        items
            .iter()
            .find(|b| b.internal_name == "minecraft:sea_pickle"),
    ) {
        assert_eq!(
            sea_pickle_block.max_state_id - sea_pickle_block.min_state_id,
            4
        );

        for state in sea_pickle_block.min_state_id..=sea_pickle_block.max_state_id {
            // assuming num pickles is the least significant state number
            let num_drops = (state - sea_pickle_block.min_state_id) as usize % 4;
            registry.set_block_drops(
                BlockWithState::from_id(state),
                None,
                None,
                Some(DropTable::Single(ItemDrop {
                    item: Item::from_id(sea_pickle_item.id),
                    quantity: ItemDropQuantity::FixedMultiple(num_drops),
                })),
            );
        }
    }
}

fn handle_leaves(blocks: &Vec<json::Block>, items: &Vec<json::Item>, registry: &mut BlockDropRegistry) {
    // leaves follow a table of probabilities:
    // | Item                | Source                              | No fortune | Fortune 1 | Fortune 2 | Fortune 3 |
    // |---------------------|-------------------------------------|------------|-----------|-----------|-----------|
    // | Jungle Saplings     | Jungle Leaves                       | 1/40       | 1/36      | 1/32      | 1/24      |
    // | Saplings and Azalea | Azalea Leaves or other Leaves       | 1/20       | 1/16      | 1/12      | 1/10      |
    // | 1-2 Sticks          | All Leaves                          | 1/50       | 1/45      | 1/40      | 1/30      |
    // | Apples              | Oak and Dark Oak Leaves             | 1/200      | 1/180     | 1/160     | 1/120     |
    let leaves = blocks
        .iter()
        .filter(|b| b.internal_name.ends_with("_leaves"));
    for leaf_block in leaves {
        let leaf_item = items
            .iter()
            .find(|b| b.internal_name == leaf_block.internal_name)
            .expect(format!("Item form of {} not found", leaf_block.internal_name).as_str());

        let shears = items
            .iter()
            .find(|b| b.internal_name == "minecraft:shears")
            .expect("Shears not found");
        registry.set_tools(
            Block::from_id(leaf_block.id),
            vec![Item::from_id(shears.id)],
        );

        // with silk touch or with shears
        let with_tool = DropTable::Single(ItemDrop {
            item: Item::from_id(leaf_item.id),
            quantity: ItemDropQuantity::Single,
        });

        let sapling_drop = {
            let name = if leaf_block.internal_name.ends_with("azalea_leaves") {
                leaf_block.internal_name.replace("_leaves", "")
            } else {
                leaf_block.internal_name.replace("_leaves", "_sapling")
            };
            let item = items
                .iter()
                .find(|b| b.internal_name == name)
                .expect(format!("{} not found", name).as_str());

            let chance = if leaf_block.internal_name.ends_with("jungle_leaves") {
                [1.0 / 40.0, 1.0 / 36.0, 1.0 / 32.0, 1.0 / 24.0, 1.0 / 24.0]
            } else {
                [1.0 / 20.0, 1.0 / 16.0, 1.0 / 12.0, 1.0 / 10.0, 1.0 / 10.0]
            };

            ItemDrop {
                item: Item::from_id(item.id),
                quantity: ItemDropQuantity::ChanceFromTable {
                    chance,
                },
            }
        };

        let stick_drop = {
            let item = items
                .iter()
                .find(|b| b.internal_name == "minecraft:stick")
                .expect("sticks not found");
            let chance = [1.0 / 50.0, 1.0 / 45.0, 1.0 / 40.0, 1.0 / 30.0, 1.0 / 30.0];

            ItemDrop {
                item: Item::from_id(item.id),
                quantity: ItemDropQuantity::ChanceFromTable {
                    chance,
                },
            }
        };

        let with_hands = if leaf_block.internal_name.ends_with("oak_leaves") {
            let apple_drop = {
                let item = items
                    .iter()
                    .find(|b| b.internal_name == "minecraft:apple")
                    .expect("apples not found");
                let chance = [1.0 / 200.0, 1.0 / 180.0, 1.0 / 160.0, 1.0 / 120.0, 1.0 / 120.0];

                ItemDrop {
                    item: Item::from_id(item.id),
                    quantity: ItemDropQuantity::ChanceFromTable {
                        chance,
                    },
                }
            };

            DropTable::MultipleIndependent(vec![
                sapling_drop,
                stick_drop,
                apple_drop
            ])
        } else {
            DropTable::MultipleIndependent(vec![
                sapling_drop,
                stick_drop
            ])
        };

        registry.set_block_drops(
            BlockWithState::from_id(leaf_block.id),
            Some(with_tool),
            Some(with_hands),
            None,
        );
    }
}

fn to_drop_table(mut drops: Vec<ItemDrop>) -> Option<DropTable> {
    match drops.len() {
        0 => None,
        1 => Some(DropTable::Single(drops.pop().unwrap())),
        _ => Some(DropTable::MultipleIndependent(drops)),
    }
}
