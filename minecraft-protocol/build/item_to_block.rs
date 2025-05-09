use crate::json::{Block, BlockDropMapping};
use crate::{categories, string_distance};
use convert_case::{Case, Casing};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub fn generate_item_to_block_enum(block_drops: &Vec<BlockDropMapping>, blocks: &Vec<Block>) {
    let mut item_to_block: HashMap<&str, &str> = HashMap::new();

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
            if let Some(&existing) = item_to_block.get(drop.item_internal_name.as_str()) {
                let distance_checker = string_distance::Levenshtein::with_max_distance(100);
                let existing_distance =
                    distance_checker.str_distance(existing, &mapping.block_internal_name);
                let new_distance = distance_checker
                    .str_distance(&drop.item_internal_name, &mapping.block_internal_name);

                if new_distance < existing_distance {
                    item_to_block.insert(&drop.item_internal_name, &mapping.block_internal_name);
                }
            } else {
                item_to_block.insert(&drop.item_internal_name, &mapping.block_internal_name);
            }
        }
    }

    // exceptions
    item_to_block.insert("bamboo", "bamboo_sapling");

    let mut unknown_states = String::new();

    let mut item_to_block_enum = String::new();
    for (&item, &block) in item_to_block.iter() {
        let block_variant_name = block.from_case(Case::Snake).to_case(Case::UpperCamel);
        let item_variant_name = item.from_case(Case::Snake).to_case(Case::UpperCamel);

        let block = blocks.iter().find(|b| &b.internal_name == block).unwrap();

        if block.states.is_empty() {
            item_to_block_enum.push_str(&format!(
                "\t\t\tItem::{item_variant_name} => Some(BlockWithState::{block_variant_name}),\n"
            ));
        } else {
            item_to_block_enum.push_str(&format!(
                "\t\t\tItem::{item_variant_name} => Some(BlockWithState::{block_variant_name} {{\n"
            ));

            for state in &block.states {
                if state.name == "waterlogged" {
                    item_to_block_enum
                        .push_str("\t\t\t\twaterlogged: is_waterlogged(placed_on),\n");
                } else if state.name == "axis" {
                    // logs, basalt, etc.
                    item_to_block_enum
                        .push_str("\t\t\t\taxis: to_axis(facing_x, facing_y, facing_z),\n");
                } else if state.name == "axis_horizontal" {
                    // portals
                    item_to_block_enum
                        .push_str("\t\t\t\taxis: to_axis_horizontal(facing_x, facing_z),\n");
                } else if state.name == "facing" {
                    item_to_block_enum
                        .push_str("\t\t\t\tfacing: to_facing(facing_x, facing_y, facing_z),\n");
                } else if state.name == "facing_hopper" {
                    // hoppers
                    item_to_block_enum.push_str(
                        "\t\t\t\tfacing_hopper: to_hopper_facing(facing_x, facing_y, facing_z),\n",
                    );
                } else if state.name == "facing_horizontal" {
                    // doors, fences
                    item_to_block_enum.push_str(
                        "\t\t\t\tfacing_horizontal: to_facing_horizontal(-facing_x, -facing_z),\n",
                    );
                } else if state.name == "face" {
                    // buttons, grindstone
                    item_to_block_enum.push_str(
                        "\t\t\t\tface: to_floor_wall_ceiling(cursor_position_vertical),\n",
                    );
                } else if state.name == "half" {
                    // slabs, stairs, doors
                    item_to_block_enum.push_str(
                        "\t\t\t\thalf: if cursor_position_vertical > 0.5 { Half::Top } else { Half::Bottom },\n"
                    );
                } else if state.name == "vertical_half" {
                    // probably a plant
                    item_to_block_enum.push_str("\t\t\t\tvertical_half: VerticalHalf::Lower,\n");
                } else if state.name == "rotation" {
                    item_to_block_enum
                        .push_str("\t\t\t\trotation: to_sign_rotation(facing_x, facing_z),\n");
                } else if state.name == "hinge" {
                    // TODO calculate hinge position
                    item_to_block_enum.push_str("\t\t\t\thinge: Hinge::Left,\n");
                } else if state.name == "candles" {
                    // adding a candle must be handled earlier
                    item_to_block_enum.push_str("\t\t\t\tcandles: 0,\n");
                } else if state.name == "shape" {
                    if categories::is_stairs(item) {
                        item_to_block_enum.push_str(" \t\t\t\tshape: StairsShape::Straight,\n");
                    } else if categories::is_rail(item) {
                        item_to_block_enum.push_str(" \t\t\t\tshape: RailShape::NorthSouth,\n");
                    } else {
                        unknown_states.push_str(&format!("{}: {:?}\n", block.internal_name, state));
                    }
                } else if state.name == "ty" || state.name == "type" {
                    if categories::is_slab(item) {
                        item_to_block_enum.push_str(
                            "\t\t\t\tty: if cursor_position_vertical > 0.5 { SlabType::Top } else { SlabType::Bottom },\n"
                        );
                    } else if categories::is_chest(item) {
                        item_to_block_enum.push_str("\t\t\t\tty: ChestType::Single,\n");
                    } else {
                        unknown_states.push_str(&format!("{}: {:?}\n", block.internal_name, state));
                    }
                } else if state.name == "thickness" {
                    // dripstone
                    item_to_block_enum.push_str("\t\t\t\tthickness: Thickness::Tip,\n");
                } else if state.name == "vertical_direction" {
                    // dripstone
                    item_to_block_enum.push_str("\t\t\t\tvertical_direction: VerticalDirection::Up,\n");
                } else if state.name == "part" {
                    // beds
                    item_to_block_enum.push_str("\t\t\t\tpart: Part::Foot,\n");
                } else if state.name == "tilt" {
                    // dripleaf
                    item_to_block_enum.push_str("\t\t\t\ttilt: Tilt::None,\n");
                } else if state.name == "mode" && item == "comparator" {
                    item_to_block_enum.push_str("\t\t\t\tmode: ComparatorMode::Compare,\n");
                } else if state.name == "attachment" {
                    item_to_block_enum.push_str("\t\t\t\tattachment: Attachment::Floor,\n");
                } else if state.name == "sculk_sensor_phase" {
                    item_to_block_enum.push_str("\t\t\t\tsculk_sensor_phase: SculkSensorPhase::Inactive,\n");
                } else if state.name == "leaves" {
                    item_to_block_enum.push_str("\t\t\t\tleaves: Leaves::None,\n");
                } else if state.name == "instrument" {
                    item_to_block_enum.push_str("\t\t\t\tinstrument: Instrument::Harp,\n");
                } else if state.ty == "bool" {
                    // blanket value for all booleans
                    item_to_block_enum.push_str(&format!("\t\t\t\t{}: false,\n", state.name));
                } else if state.ty == "int" {
                    // blanket value for all integers
                    item_to_block_enum.push_str(&format!("\t\t\t\t{}: 0,\n", state.name));
                }
            }

            if item == "redstone" {
                item_to_block_enum.push_str(
                    "\t\t\t\teast: WireEast::None,\n\
                    \t\t\t\tnorth: WireNorth::None,\n\
                    \t\t\t\tsouth: WireSouth::None,\n\
                    \t\t\t\twest: WireWest::None,\n",
                );
            } else if categories::is_wall(item) {
                item_to_block_enum.push_str(
                    "\t\t\t\tnorth: WallNorth::None,\n\
                    \t\t\t\teast: WallEast::None,\n\
                    \t\t\t\tsouth: WallSouth::None,\n\
                    \t\t\t\twest: WallWest::None,\n",
                );
            }

            item_to_block_enum.push_str("\t\t\t}),\n");
        }
    }

    if !unknown_states.is_empty() {
        File::create("build/debug.json")
            .unwrap()
            .write_all(unknown_states.as_bytes())
            .unwrap();
    } else {
        std::fs::remove_file("build/debug.json").ok();
    }

    // Generate the code
    let code = format!(
        r#"//! Contains the [item_to_block] function to help with
//! block placement.

use crate::{{
    ids::{{blocks::Block, items::Item, block_states::*}},
    components::{{blocks::BlockFace, item_placement::*}}
}};

pub fn item_to_block(
    item: Item,
    placed_on_face: BlockFace,
    placed_on: BlockWithState,
    facing_x: f32,
    facing_y: f32,
    facing_z: f32,
    cursor_position_horizontal: f32,
    cursor_position_vertical: f32,
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
