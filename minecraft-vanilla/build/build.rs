mod blocks;
mod entities;
mod items;
mod recipes;
mod block_drops;

use minecraft_external::game_data;
use std::fs::File;
use std::io::Write;

fn main() {
    let target = std::env::var("OUT_DIR").expect("Set CARGO_TARGET_DIR to the target directory");

    let file_locations = game_data::get_file_locations(&target);
    let items = game_data::get_items(&target, &file_locations);
    let blocks = game_data::get_blocks(&target, &file_locations);
    let entities = game_data::get_entities(&target, &file_locations);
    let item_recipes = game_data::get_recipes(&target, &file_locations);
    let block_drops = game_data::get_block_drops(&target, &file_locations);

    std::fs::create_dir_all("data").unwrap();

    let mut items_rs = File::create("src/ids/items.rs").unwrap();
    items::generate_item_enum(&items, &mut items_rs);

    let mut entities_rs = File::create("src/ids/entities.rs").unwrap();
    entities::generate_entity_enum(&entities, &mut entities_rs);

    let mut blocks_rs = File::create("src/ids/blocks.rs").unwrap();
    blocks::generate_block_enum(&blocks, &mut blocks_rs);

    let mut block_states_rs = File::create("src/ids/block_states.rs").unwrap();
    blocks::generate_block_with_state_enum(&blocks, &mut block_states_rs);

    let mut recipes_bin = File::create("data/recipes.bin").unwrap();
    let recipes_registry = recipes::get_recipes_registry(item_recipes);
    let encoded = bincode::serde::encode_to_vec(&recipes_registry, bincode::config::standard())
        .expect("Failed to encode recipes");
    recipes_bin.write_all(&encoded).unwrap();
    drop(recipes_bin);

    let mut blocks_bin = File::create("data/blocks.bin").unwrap();
    let block_registry = blocks::get_block_registry(&blocks);
    let encoded = bincode::serde::encode_to_vec(&block_registry, bincode::config::standard())
        .expect("Failed to encode blocks");
    blocks_bin.write_all(&encoded).unwrap();
    drop(blocks_bin);

    let mut block_drops_bin = File::create("data/block_drops.bin").unwrap();
    let drop_registry = block_drops::get_block_drop_registry(&block_registry, &block_drops, &blocks, &items);

    let encoded = bincode::serde::encode_to_vec(&drop_registry, bincode::config::standard())
        .expect("Failed to encode block drops");
    block_drops_bin.write_all(&encoded).unwrap();
    drop(block_drops_bin);
}
