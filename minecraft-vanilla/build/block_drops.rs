use minecraft_external::json;
use minecraft_game_logic::block_drop_registry::*;
use minecraft_protocol::data::{blocks::Block, items::Item};
use std::fs::File;
use std::io::Write;

// Currently impossible to encode:
//  * sea pickles have "stackSizeRange": [4, 4] and state data with the name "pickles" that specify how much must be dropped
//  * every type of candle has "stackSizeRange": [4, 4] and state data with the name "candles" that specify how much must be dropped
//  * beehive drop bee entities (if there are bees in the hive, which is not part of the state data)

pub fn generate_block_drop_binary(
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

        registry.add_tools(Block::from_id(block.id), &tools);

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
                if min == None || min == Some(-1) {
                    // TODO drop quantity depends on exact block state
                    ItemDropQuantity::FixedMultiple(1)
                } else if min == max {
                    ItemDropQuantity::Single
                } else if min.is_some() && max.is_some() {
                    ItemDropQuantity::RandomRange { min: min.unwrap() as usize, max: max.unwrap() as usize }
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

        registry.set_block_drops(Block::from_id(block.id), with_tool, with_hands, with_silk_touch);
    }

    // handle special cases


    todo!();

    let encoded = bincode::serde::encode_to_vec(&registry, bincode::config::standard())
        .expect("Failed to encode recipes");

    file.write_all(&encoded).unwrap()
}

fn to_drop_table(mut drops: Vec<ItemDrop>) -> Option<DropTable> {
    match drops.len() {
        0 => None,
        1 => Some(DropTable::Single(drops.pop().unwrap())),
        _ => Some(DropTable::MultipleIndependent(drops))
    }
}
