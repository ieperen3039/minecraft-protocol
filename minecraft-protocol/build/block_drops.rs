use crate::json::{Block, BlockDropMapping, BlockItemDrop};
use convert_case::{Case, Casing};
use std::fs::File;
use std::io::Write;

pub fn generate_block_drop_enum(blocks: &Vec<Block>, block_drops: &Vec<BlockDropMapping>) {
    let mut block_to_item_enum = String::new();

    for mapping in block_drops {
        if mapping.drops.is_empty() {
            continue;
        }

        let block = blocks
            .iter()
            .find(|block| block.internal_name == mapping.block_internal_name)
            .unwrap();

        let block_variant_name = mapping
            .block_internal_name
            .from_case(Case::Snake)
            .to_case(Case::UpperCamel);

        let mut silk_touch_drop = Vec::new();
        let mut tool_drops = Vec::new();
        let mut hand_drops = Vec::new();

        for drop in mapping.drops.iter() {
            if drop.silk_touch == Some(true) {
                silk_touch_drop.push(drop);
            } else if !block.harvest_tools.is_empty() {
                tool_drops.push(drop);
            } else {
                hand_drops.push(drop);
            }
        }

        append_drops(
            &mut block_to_item_enum,
            &block_variant_name,
            "if silk_touch",
            silk_touch_drop,
        );

        let tool_ids: Vec<u32> = block.harvest_tools.keys().cloned().collect();
        append_drops(
            &mut block_to_item_enum,
            &block_variant_name,
            &format!("if {tool_ids:?}.contains(&tool)"),
            tool_drops,
        );

        append_drops(&mut block_to_item_enum, &block_variant_name, "", hand_drops);
    }

    // Generate the code
    let code = format!(
        r#"//! Contains the [block_to_item] function to help with
//! block breaking

use crate::{{
    ids::{{blocks::Block, items::Item}},
    nbt::{{NbtTag, arrays::NbtList}},
}};

use std::collections::HashMap;
use rand::Rng;


pub fn block_to_item<R: Rng + ?Sized>(
    block: Block,
    tool: Option<Item>,
    nbt: Option<NbtTag>,
    rng: &mut R,
) -> Vec<Item> {{
    match (tool, nbt) {{
        (None, _) => block_to_item_inner(block, 0, false, 0, rng),
        (Some(tool), Some(NbtTag::Compound(tags))) => {{
            if let Some(NbtTag::List(NbtList::Compound(enchantments))) = tags.get("Enchantments") {{
                let mut enchantment_ids: HashMap<&str, i16> = HashMap::new();
                for e in enchantments {{
                    match (e.get("id"), e.get("lvl")) {{
                        (Some(NbtTag::String(id)), Some(NbtTag::Short(lvl))) => {{
                            enchantment_ids.insert(id.as_str(), *lvl);
                        }},
                        (Some(NbtTag::String(id)), None) => {{
                            enchantment_ids.insert(id.as_str(), 1);
                        }},
                        _ => {{}},
                    }}
                }}

                let silk_touch = enchantment_ids.contains_key(&"silk_touch");
                let fortune: i16 = enchantment_ids.get(&"fortune").cloned().unwrap_or(0);

                block_to_item_inner(block, tool as u32, silk_touch, fortune as u32, rng)
            }} else {{
                block_to_item_inner(block, tool as u32, false, 0, rng)
            }}
        }},
        (Some(tool), _) => block_to_item_inner(block, tool as u32, false, 0, rng),
    }}
}}

fn block_to_item_inner<R: Rng + ?Sized>(
    block: Block,
    tool: u32,
    silk_touch: bool,
    fortune_level: u32,
    rng: &mut R,
) -> Vec<Item> {{
    match block {{
{block_to_item_enum}
        // all other blocks do not drop anything
        _ => vec![]
    }}
}}
"#
    );

    File::create("src/components/block_drops.rs")
        .unwrap()
        .write_all(code.as_bytes())
        .unwrap()
}

fn append_drops(
    block_to_item_enum: &mut String,
    block_variant_name: &str,
    condition: &str,
    drops: Vec<&BlockItemDrop>,
) {
    if drops.len() == 1 {
        let drop = drops[0];
        let item_variant_name = drop
            .item_internal_name
            .from_case(Case::Snake)
            .to_case(Case::UpperCamel);

        let minimum_count = drop.stack_size_range[0].unwrap_or(1);
        let maximum_count = drop.stack_size_range[1].unwrap_or(minimum_count);

        // we ignore drop chance, because that data is broken
        let tool_drop_string = if minimum_count != maximum_count {
            format!(
                "{{\n\
                \t\t\tlet max_num_drops = {maximum_count} + fortune_level as usize;\n\
                \t\t\tlet num_drops = rng.random_range({minimum_count}..max_num_drops);\n\
                \t\t\tvec![Item::{item_variant_name}; num_drops]\n\
                \t\t}}"
            )
        } else if maximum_count > 1 {
            format!("vec![Item::{item_variant_name}; {maximum_count}]")
        } else {
            format!("vec![Item::{item_variant_name}]")
        };

        block_to_item_enum.push_str(
            &format!("\t\tBlock::{block_variant_name} {condition} => {tool_drop_string},\n"),
        );
    } else if drops.len() > 1 {
        block_to_item_enum
            .push_str(&format!("\t\tBlock::{block_variant_name} {condition} => "));

        // if any of the drops is randomized, we need to generate a multiline vector construction
        let has_randomized_drops = drops.iter().any(|drop| {
            drop.stack_size_range[0].is_some()
                && drop.stack_size_range[1].is_some()
                && drop.stack_size_range[0] != drop.stack_size_range[1]
        });

        if has_randomized_drops {
            append_multiline_drops(block_to_item_enum, &drops);
        } else {
            block_to_item_enum.push_str("vec![");

            for drop in drops {
                let item_variant_name = drop
                    .item_internal_name
                    .from_case(Case::Snake)
                    .to_case(Case::UpperCamel);

                let count = drop.stack_size_range[0].unwrap_or(1);
                for _ in 0..count {
                    block_to_item_enum.push_str(&format!("Item::{item_variant_name}, "));
                }
            }

            block_to_item_enum.truncate(block_to_item_enum.len() - 2);
            block_to_item_enum.push_str("],\n");
        }
    }
}

fn append_multiline_drops(block_to_item_enum: &mut String, drops: &Vec<&BlockItemDrop>) {
    block_to_item_enum.push_str(
        "{\n\
            \t\t\tlet mut items = Vec::new();\n",
    );

    for drop in drops {
        let item_variant_name = drop
            .item_internal_name
            .from_case(Case::Snake)
            .to_case(Case::UpperCamel);

        let minimum_count = drop.stack_size_range[0].unwrap_or(1);
        let maximum_count = drop.stack_size_range[1].unwrap_or(minimum_count);

        if minimum_count != maximum_count {
            block_to_item_enum.push_str(
                &format!(
                    "\t\t\tlet max_num_drops = {maximum_count} + fortune_level as usize;\n\
                    \t\t\tfor _ in 0..rng.random_range({minimum_count}..max_num_drops) {{\n\
                    \t\t\t\titems.push(Item::{item_variant_name});\n\
                    \t\t\t}}\n"
                ),
            );
        } else if maximum_count > 1 {
            block_to_item_enum.push_str(
                &format!(
                    "\t\t\tfor _ in 0..{maximum_count} {{\n \
                    \t\t\t\titems.push(Item::{item_variant_name});\n \
                    \t\t\t}}\n "
                ),
            );
        } else {
            block_to_item_enum
                .push_str(&format!("\t\t\titems.push(Item::{item_variant_name});\n"));
        }
    }

    block_to_item_enum.push_str("\t\t\titems\n\t\t}\n");
}
