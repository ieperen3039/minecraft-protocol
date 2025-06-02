mod blocks;
mod entities;
mod items;

use minecraft_external::game_data::*;

fn main() {
    let target = std::env::var("OUT_DIR").expect("Set CARGO_TARGET_DIR to the target directory");

    println!(
        "cargo:rerun-if-changed={target}/cache-file-location-{}.json",
        VERSION
    );
    println!("cargo:rerun-if-changed=build");

    let file_locations = get_file_locations(&target);
    let items = get_items(&target, &file_locations);
    let blocks = get_blocks(&target, &file_locations);
    let entities = get_entities(&target, &file_locations);

    items::generate_item_enum(&items);
    entities::generate_entity_enum(&entities);
    blocks::generate_block_enum(&blocks);
    blocks::generate_block_with_state_enum(&blocks);
}