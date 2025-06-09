use minecraft_external::json;
use minecraft_game_logic::block_item_interactions::*;
use minecraft_protocol::data::{blocks::Block, items::Item};
use std::fs::File;
use std::io::Write;

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
            .filter_map(|(tool, is_tool)| if *is_tool { Some(tool) } else { None })
            .map(Item::from_id)
            .collect();

        registry.add_tools(Block::from_id(block.id), &tools);

        let mut with_silk_touch: Vec<ItemDrop> = Vec::new();
        let mut with_tool: Vec<ItemDrop> = Vec::new();
        let mut with_hands: Vec<ItemDrop> = Vec::new();

        for drop_entry in drop_mapping.drops {
            let item = items
                .iter()
                .find(|i| i.internal_name == drop_entry.item_internal_name);

            // TODO drop_entry.drop_chance is broken
            let quantity = {
                // TODO this must be far more advanced
                let (min, max) = drop_entry.stack_size_range;
                if min == -1 {
                    // drop quantity depends on item metadata
                    ItemDropQuantity::FixedMultiple(1)
                } else if min == max {
                    ItemDropQuantity::Single
                } else {
                    ItemDropQuantity::RandomRange { min, max }
                };
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

        registry.set_block_drops(blocks, with_tool, with_hands, with_silk_touch);
    }

    todo!();

    let encoded = bincode::serde::encode_to_vec(&registry, bincode::config::standard())
        .expect("Failed to encode recipes");

    file.write_all(&encoded).unwrap()
}
