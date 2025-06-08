mod blocks;
mod entities;
mod items;
mod recipes;

use minecraft_external::game_data;
use std::fs::File;

fn main() {
    let target = std::env::var("OUT_DIR").expect("Set CARGO_TARGET_DIR to the target directory");

    let file_locations = game_data::get_file_locations(&target);
    let items = game_data::get_items(&target, &file_locations);
    let blocks = game_data::get_blocks(&target, &file_locations);
    let entities = game_data::get_entities(&target, &file_locations);
    let item_recipes = game_data::get_recipes(&target, &file_locations);

    let mut items_rs = File::create("src/ids/items.rs").unwrap();
    items::generate_item_enum(&items, &mut items_rs);

    let mut entities_rs = File::create("src/ids/entities.rs").unwrap();
    entities::generate_entity_enum(&entities, &mut entities_rs);

    let mut blocks_rs = File::create("src/ids/blocks.rs").unwrap();
    blocks::generate_block_enum(&blocks, &mut blocks_rs);

    let mut block_states_rs = File::create("src/ids/block_states.rs").unwrap();
    blocks::generate_block_with_state_enum(&blocks, &mut block_states_rs);

    std::fs::create_dir_all("data").unwrap();
    let mut recipes_bin = File::create("data/recipes.bin").unwrap();
    recipes::generate_recipes_binary(item_recipes, &mut recipes_bin);
}
