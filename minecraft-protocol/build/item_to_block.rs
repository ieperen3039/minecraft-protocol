use crate::json::BlockDropMapping;
use crate::{categories, string_distance};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub fn generate_item_to_block_enum(block_drops: &Vec<BlockDropMapping>) {
    let mut item_to_block: HashMap<&String, &String> = HashMap::new();

    // for each block, we look at what a silk touch drop gives us,
    // and generate an enum entry that is the inverse of this.
    for mapping in block_drops {
        let mut best_drop = None;

        for drop in mapping.drops.iter() {
            if drop.silk_touch == Some(true) {
                best_drop = Some(drop);
                break;
            } else {
                best_drop = Some(drop);
            }
        }

        if let Some(drop) = best_drop {
            if let Some(&existing) = item_to_block.get(&drop.item_internal_name) {
                let distance_checker = string_distance::Levenshtein::with_max_distance(100);
                let existing_distance =
                    distance_checker.str_distance(&existing, &mapping.block_internal_name);
                let new_distance = distance_checker
                    .str_distance(&drop.item_internal_name, &mapping.block_internal_name);

                if new_distance < existing_distance {
                    println!(
                        "For block {} replaced {} with {}",
                        mapping.block_internal_name, existing, drop.item_internal_name
                    );
                    item_to_block.insert(&drop.item_internal_name, &mapping.block_internal_name);
                }
            } else {
                item_to_block.insert(&drop.item_internal_name, &mapping.block_internal_name);
            }
        }
    }

    let mut item_to_block_enum = String::new();
    for (&item, &block) in item_to_block.iter() {
        let block_variant_name = block.from_case(Case::Snake).to_case(Case::UpperCamel);
        let item_variant_name = item.from_case(Case::Snake).to_case(Case::UpperCamel);

        if categories::is_slab(item) {
            item_to_block_enum.push_str(&format!(
                "\t\t\tItem::{item_variant_name} => \n\
                \t\t\t\tSome(BlockWithState::{block_variant_name}{{\n\
                \t\t\t\t\tty: if cursor_position_z > 0.5 {{ SlabType::Top }} else {{ SlabType::Bottom }},\n\
                \t\t\t\t\twaterlogged: false,\n\
                \t\t\t\t}}),\n"
            ));
        } else {
            item_to_block_enum.push_str(&format!(
                "\t\t\tItem::{item_variant_name} => Some(Block::{block_variant_name}.into()),\n"
            ));
        }
    }

    // Generate the code
    let code = format!(
        r#"//! Contains the [item_to_block] function to help with
//! block placement.

use crate::{{
    ids::{{blocks::Block, items::Item, block_states::*}},
    nbt::{{NbtTag, arrays::NbtList}},
    components::blocks::BlockFace
}};

pub fn item_to_block(
    item: Item,
    face: BlockFace,
    cursor_position_x: f32,
    cursor_position_y: f32,
    cursor_position_z: f32,
) -> Option<BlockWithState> {{
    match item {{
{item_to_block_enum}
        _ => None,
    }}
}}
        "#
    );

    File::create("src/components/item_to_block.rs")
        .unwrap()
        .write_all(code.as_bytes())
        .unwrap()
}
