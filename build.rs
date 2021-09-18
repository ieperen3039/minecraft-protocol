use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use std::io::{ErrorKind, Read, Write};
use std::{collections::HashMap, fs::File};

const VERSION: &str = "1.17.1";

fn get_data(url: &str, cache: &str) -> serde_json::Value {
    match File::open(cache) {
        // The cache file is ready
        Ok(mut file) => {
            let mut data = Vec::new();
            if let Err(e) = file.read_to_end(&mut data) {
                panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, this file cannot be read. Error: {}", cache, e)
            }

            let json_text = match String::from_utf8(data) {
                Ok(json_text) => json_text,
                Err(e) => panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, this file appears to contain invalid text data. Error: {}\nNote: Deleting the file will allow the library to download it again.", cache, e),
            };

            let json = match serde_json::from_str(&json_text) {
                Ok(json) => json,
                Err(e) => panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, this file appears to contain invalid json data. Error: {}\nNote: Deleting the file will allow the library to download it again.", cache, e),
            };

            json
        }
        // The cache file needs to be downloaded
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let response = match minreq::get(url).send() {
                Ok(response) => response,
                Err(e) => panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded from `{}`. Unfortunately, we can't access this URL. Error: {}", url, e)
            };

            let json_text = match response.as_str() {
                Ok(json_text) => json_text,
                Err(e) => panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded from `{}`. Unfortunately, this file appears to contain invalid data. Error: {}", url, e),
            };

            let mut file = match File::create(cache) {
                Ok(file) => file,
                Err(e) => panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, we can't access this path. Error: {}", cache, e),
            };

            if let Err(e) = file.write_all(json_text.as_bytes()) {
                panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, we can't write to this path. Error: {}", cache, e)
            };

            let json = match serde_json::from_str(json_text) {
                Ok(json) => json,
                Err(e) => panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, this file appears to contain invalid json data. Error: {}\nNote: Deleting the file will allow the library to download it again.", cache, e),
            };

            json
        }

        // The cache file cannot be accessed
        Err(e) => {
            panic!("The minecraft-format library uses a build script to generate data structures from extracted data. The extracted data is downloaded and cached to `{}`. Unfortunately, we can't access this path. Error: {}", cache, e);
        }
    }
}

mod blocks {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct BlockState {
        name: String,
        #[serde(rename = "type")]
        ty: String,
        num_values: usize,
        values: Option<Vec<String>>,
    }

    impl BlockState {
        fn ty(&self, block_name: &str, competing_definitions: bool) -> String {
            match self.ty.as_str() {
                "int" => {
                    let values: Vec<i128> = self
                        .values
                        .as_ref()
                        .expect("No values for int block state")
                        .iter()
                        .map(|v| v.parse().expect("Invalid block state value: expected int"))
                        .collect();
                    let mut min_value: i128 = *values.first().unwrap_or(&0);
                    let mut max_value: i128 = *values.first().unwrap_or(&0);

                    for value in values {
                        if value < min_value {
                            min_value = value;
                        }
                        if value > max_value {
                            max_value = value;
                        }
                    }

                    if min_value >= u8::MIN as i128 && max_value <= u8::MAX as i128 {
                        return String::from("u8");
                    }
                    if min_value >= i8::MIN as i128 && max_value <= i8::MAX as i128 {
                        return String::from("i8");
                    }
                    if min_value >= u16::MIN as i128 && max_value <= u16::MAX as i128 {
                        return String::from("u16");
                    }
                    if min_value >= i16::MIN as i128 && max_value <= i16::MAX as i128 {
                        return String::from("i16");
                    }
                    if min_value >= u32::MIN as i128 && max_value <= u32::MAX as i128 {
                        return String::from("u32");
                    }
                    if min_value >= i32::MIN as i128 && max_value <= i32::MAX as i128 {
                        return String::from("i32");
                    }
                    if min_value >= u64::MIN as i128 && max_value <= u64::MAX as i128 {
                        return String::from("u64");
                    }
                    if min_value >= i64::MIN as i128 && max_value <= i64::MAX as i128 {
                        return String::from("i64");
                    }
                    String::from("i128")
                }
                "enum" => match competing_definitions {
                    true => format!("{}_{}", block_name, self.name),
                    false => self.name.to_string(),
                }
                .from_case(Case::Snake)
                .to_case(Case::UpperCamel),
                "bool" => String::from("bool"),
                _ => unimplemented!(),
            }
        }

        fn define_enum(&self, block_name: &str, competing_definitions: bool) -> String {
            if self.ty.as_str() != "enum" {
                panic!("Called defined enum on non-enum");
            }

            let mut variants = String::new();
            for (i, value) in self
                .values
                .as_ref()
                .expect("Expecting values in enum (state id)")
                .iter()
                .enumerate()
            {
                variants.push_str(&format!(
                    "\n\t{} = {},",
                    value.from_case(Case::Snake).to_case(Case::UpperCamel),
                    i
                ));
            }

            format!(
                r#"#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum {} {{{}
}}"#,
                self.ty(block_name, competing_definitions),
                variants
            )
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Block {
        id: u32,
        #[serde(rename = "name")]
        text_id: String,
        display_name: String,
        hardness: f32,
        resistance: f32,
        diggable: bool,
        transparent: bool,
        filter_light: u8,
        emit_light: u8,
        default_state: u32,
        min_state_id: u32,
        max_state_id: u32,
        drops: Vec<u32>,
        material: Option<String>,
        #[serde(default)]
        harvest_tools: HashMap<u32, bool>,
        states: Vec<BlockState>,
    }

    #[allow(clippy::explicit_counter_loop)]
    pub fn generate_block_enum(data: serde_json::Value) {
        let mut blocks: Vec<Block> = serde_json::from_value(data).expect("Invalid block data");
        blocks.sort_by_key(|block| block.id);

        // Look for missing blocks in the array
        let mut expected = 0;
        for block in &blocks {
            if block.id != expected {
                panic!("The block with id {} is missing.", expected)
            }
            expected += 1;
        }

        // Process a few fields
        let mut raw_harvest_tools: Vec<Vec<u32>> = Vec::new();
        let mut raw_materials: Vec<String> = Vec::new();
        for block in &blocks {
            raw_harvest_tools.push(
                block
                    .harvest_tools
                    .clone()
                    .into_iter()
                    .map(|(k, _v)| k)
                    .collect(),
            );
            let mut material = block
                .material
                .clone()
                .unwrap_or_else(|| "unknown_material".to_string())
                .split(';')
                .next()
                .unwrap()
                .to_string();
            if material.starts_with("mineable") {
                material = "unknown_material".to_string();
            }
            raw_materials.push(material.from_case(Case::Snake).to_case(Case::UpperCamel));
        }

        // Generate the MaterialBlock enum and array
        let mut different_materials = raw_materials.clone();
        different_materials.sort();
        different_materials.dedup();
        let mut material_variants = String::new();
        for material in different_materials {
            material_variants.push_str(&format!("\t{},\n", material));
        }
        let mut materials = String::new();
        materials.push('[');
        for material in raw_materials {
            materials.push_str("Some(BlockMaterial::");
            materials.push_str(&material);
            materials.push_str("), ");
        }
        materials.push(']');

        // Generate the HARVEST_TOOLS array
        let mut harvest_tools = String::new();
        harvest_tools.push('[');
        for block_harvest_tools in raw_harvest_tools {
            harvest_tools.push_str("&[");
            for harvest_tool in block_harvest_tools {
                harvest_tools.push_str(&harvest_tool.to_string());
                harvest_tools.push_str(", ");
            }
            harvest_tools.push_str("], ");
        }
        harvest_tools.push(']');

        // Enumerate the air blocks
        let mut air_blocks = vec![false; expected as usize];
        for air_block in &[
            "air",
            "cave_air",
            "grass",
            "torch",
            "wall_torch",
            "wheat",
            "soul_torch",
            "soul_wall_torch",
            "carrots",
            "potatoes",
        ] {
            let mut success = false;
            for block in &blocks {
                if &block.text_id.as_str() == air_block {
                    air_blocks[block.id as usize] = true;
                    success = true;
                    break;
                }
            }
            if !success {
                panic!("Could not find block {} in the block array", air_block);
            }
        }

        // Generate the variants of the Block enum
        let mut variants = String::new();
        for block in &blocks {
            let name = block
                .text_id
                .from_case(Case::Snake)
                .to_case(Case::UpperCamel);
            variants.push_str(&format!("\t{} = {},\n", name, block.id));
        }

        // Generate the `match` of state ids
        let mut state_id_match_arms = String::new();
        for block in &blocks {
            let name = block
                .text_id
                .from_case(Case::Snake)
                .to_case(Case::UpperCamel);
            let start = block.min_state_id;
            let stop = block.max_state_id;
            if start != stop {
                state_id_match_arms.push_str(&format!(
                    "\t\t\t{}..={} => Some(Block::{}),\n",
                    start, stop, name
                ));
            } else {
                state_id_match_arms
                    .push_str(&format!("\t\t\t{} => Some(Block::{}),\n", start, name));
            }
        }

        // Generate the code
        let code = format!(
            r#"use crate::*;

/// See [implementations](#implementations) for useful methods.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Block {{
{variants}
}}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockMaterial {{
{material_variants}
}}

impl Block {{
    #[inline]
    pub fn from_id(id: u32) -> Option<Block> {{
        if id < {max_value} {{
            Some(unsafe{{std::mem::transmute(id)}})
        }} else {{
            None
        }}
    }}

    pub fn from_state_id(state_id: u32) -> Option<Block> {{
        match state_id {{
{state_id_match_arms}
            _ => None,
        }}
    }}

    /// Get the textual identifier of this block.
    #[inline]
    pub fn text_id(self) -> &'static str {{
        unsafe {{*TEXT_IDS.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn default_state_id(self) -> u32 {{
        unsafe {{*DEFAULT_STATE_IDS.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn id(self) -> u32 {{
        self as u32
    }}

    /// This returns the item that will be dropped if you break the block.
    /// If the item is Air, there is actually no drop.
    #[inline]
    pub fn associated_item_id(self) -> u32 {{
        unsafe {{*ITEM_IDS.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn resistance(self) -> f32 {{
        unsafe {{*RESISTANCES.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn hardness(self) -> f32 {{
        unsafe {{*HARDNESSES.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn material(self) -> Option<BlockMaterial> {{
        unsafe {{*MATERIALS.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn display_name(self) -> &'static str {{
        unsafe {{*DISPLAY_NAMES.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn state_id_range(self) -> std::ops::Range<u32> {{
        unsafe {{STATE_ID_RANGES.get_unchecked((self as u32) as usize).clone()}}
    }}

    #[inline]
    pub fn is_diggable(self) -> bool {{
        unsafe {{*DIGGABLE.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn is_transparent(self) -> bool {{
        unsafe {{*TRANSPARENT.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn compatible_harvest_tools(self) -> &'static [u32] {{
        unsafe {{*HARVEST_TOOLS.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn light_emissions(self) -> u8 {{
        unsafe {{*LIGHT_EMISSIONS.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn light_absorption(self) -> u8 {{
        unsafe {{*LIGHT_ABSORPTION.get_unchecked((self as u32) as usize)}}
    }}

    /// A "air block" is a block on which a player cannot stand, like air, wheat, torch...
    /// Fire is excluded since you may not want your clients to walk trought fire by default.
    /// The list of air blocks is maintained by hand.
    /// It could not be exhaustive.
    /// See also [Block::is_blocking].
    #[inline]
    pub fn is_air_block(self) -> bool {{
        unsafe {{*AIR_BLOCKS.get_unchecked((self as u32) as usize)}}
    }}

    /// The opposite of [Block::is_air_block].
    /// Fire is included since you may not want your clients to walk trought fire by default.
    /// The list of blocking blocks is maintained by hand.
    /// It could not be exhaustive.
    #[inline]
    pub fn is_blocking(self) -> bool {{
        unsafe {{!(*AIR_BLOCKS.get_unchecked((self as u32) as usize))}}
    }}
}}

impl From<super::block_states::BlockWithState> for Block {{
    #[inline]
    fn from(block_with_state: super::block_states::BlockWithState) -> Block {{
        unsafe {{std::mem::transmute(block_with_state.block_id())}}
    }}
}}

impl<'a> MinecraftPacketPart<'a> for Block {{
    #[inline]
    fn serialize_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {{
        VarInt((self as u32) as i32).serialize_minecraft_packet_part(output)
    }}

    #[inline]
    fn deserialize_minecraft_packet_part(input: &'a[u8]) -> Result<(Self, &'a[u8]), &'static str> {{
        let (id, input) = VarInt::deserialize_minecraft_packet_part(input)?;
        let id = std::cmp::max(id.0, 0) as u32;
        let block = Block::from_id(id).ok_or("No block corresponding to the specified numeric ID.")?;
        Ok((block, input))
    }}
}}

const TEXT_IDS: [&str; {max_value}] = {text_ids:?};
const DISPLAY_NAMES: [&str; {max_value}] = {display_names:?};
const STATE_ID_RANGES: [std::ops::Range<u32>; {max_value}] = {state_id_ranges:?};
const DEFAULT_STATE_IDS: [u32; {max_value}] = {default_state_ids:?};
const ITEM_IDS: [u32; {max_value}] = {item_ids:?};
const RESISTANCES: [f32; {max_value}] = {resistances:?};
const MATERIALS: [Option<BlockMaterial>; {max_value}] = {materials};
const HARVEST_TOOLS: [&[u32]; {max_value}] = {harvest_tools};
const HARDNESSES: [f32; {max_value}] = {hardnesses:?};
const LIGHT_EMISSIONS: [u8; {max_value}] = {light_emissions:?};
const LIGHT_ABSORPTION: [u8; {max_value}] = {light_absorption:?};
const DIGGABLE: [bool; {max_value}] = {diggable:?};
const TRANSPARENT: [bool; {max_value}] = {transparent:?};
const AIR_BLOCKS: [bool; {max_value}] = {air_blocks:?};
"#,
            variants = variants,
            material_variants = material_variants,
            max_value = expected,
            state_id_match_arms = state_id_match_arms,
            text_ids = blocks.iter().map(|b| &b.text_id).collect::<Vec<_>>(),
            display_names = blocks.iter().map(|b| &b.display_name).collect::<Vec<_>>(),
            state_id_ranges = blocks
                .iter()
                .map(|b| b.min_state_id..b.max_state_id + 1)
                .collect::<Vec<_>>(),
            default_state_ids = blocks.iter().map(|b| b.default_state).collect::<Vec<_>>(),
            item_ids = blocks
                .iter()
                .map(|b| b.drops.get(0).copied().unwrap_or(0))
                .collect::<Vec<_>>(),
            materials = materials,
            resistances = blocks.iter().map(|b| b.resistance).collect::<Vec<_>>(),
            harvest_tools = harvest_tools,
            hardnesses = blocks.iter().map(|b| b.hardness).collect::<Vec<_>>(),
            light_emissions = blocks.iter().map(|b| b.emit_light).collect::<Vec<_>>(),
            light_absorption = blocks.iter().map(|b| b.filter_light).collect::<Vec<_>>(),
            diggable = blocks.iter().map(|b| b.diggable).collect::<Vec<_>>(),
            transparent = blocks.iter().map(|b| b.transparent).collect::<Vec<_>>(),
            air_blocks = air_blocks,
        );

        File::create("src/ids/blocks.rs")
            .unwrap()
            .write_all(code.as_bytes())
            .unwrap()
    }

    #[allow(clippy::explicit_counter_loop)]
    pub fn generate_block_with_state_enum(data: serde_json::Value) {
        let mut blocks: Vec<Block> = serde_json::from_value(data).expect("Invalid block data");
        blocks.sort_by_key(|block| block.id);

        // Look for missing blocks in the array
        let mut expected = 0;
        for block in &blocks {
            if block.id != expected {
                panic!("The block with id {} is missing.", expected)
            }
            expected += 1;
        }

        // Generate the enum definitions
        let mut enum_definitions = Vec::new();
        let mut enum_definitions_string = String::new();
        let mut already_defined_enums = Vec::new();
        for block in &blocks {
            for state in &block.states {
                if state.ty.as_str() == "enum" {
                    enum_definitions.push((&block.text_id, state));
                }
            }
        }
        for (block_name, enum_definition) in &enum_definitions {
            let mut competing_definitions = false;
            for (_, enum_definition2) in &enum_definitions {
                if enum_definition.name == enum_definition2.name
                    && enum_definition.values != enum_definition2.values
                {
                    competing_definitions = true;
                    break;
                }
            }
            if !already_defined_enums
                .contains(&enum_definition.ty(block_name, competing_definitions))
            {
                enum_definitions_string
                    .push_str(&enum_definition.define_enum(block_name, competing_definitions));
                enum_definitions_string.push('\n');
                enum_definitions_string.push('\n');

                already_defined_enums.push(enum_definition.ty(block_name, competing_definitions));
            }
        }

        // Generate the variants of the Block enum
        let mut variants = String::new();
        for block in &blocks {
            let name = block
                .text_id
                .from_case(Case::Snake)
                .to_case(Case::UpperCamel);
            let mut fields = String::new();
            for state in &block.states {
                let name = match state.name.as_str() == "type" {
                    true => "ty",
                    false => state.name.as_str(),
                };
                let competing_definitions =
                    already_defined_enums.contains(&state.ty(&block.text_id, true));
                fields.push_str(&format!(
                    "{}: {}, ",
                    name,
                    state.ty(&block.text_id, competing_definitions)
                ));
            }
            if fields.is_empty() {
                variants.push_str(&format!("\t{},\n", name));
            } else {
                variants.push_str(&format!("\t{}{{ {}}},\n", name, fields));
            }
        }

        // Generate the `match` of state ids
        let mut state_id_match_arms = String::new();
        for block in &blocks {
            let name = block
                .text_id
                .from_case(Case::Snake)
                .to_case(Case::UpperCamel);
            let start = block.min_state_id;
            let stop = block.max_state_id;

            if block.states.is_empty() {
                state_id_match_arms.push_str(&format!(
                    "\n\t\t\t{} => Some(BlockWithState::{}),",
                    start, name
                ));
                continue;
            }

            let mut state_calculations = String::new();
            let mut fields = String::new();
            for (i, state) in block.states.iter().enumerate().rev() {
                let competing_definitions =
                    already_defined_enums.contains(&state.ty(&block.text_id, true));
                let ty = state.ty(&block.text_id, competing_definitions);
                let name = match state.name.as_str() {
                    "type" => "ty",
                    _ => &state.name,
                };
                fields.push_str(&format!("{}, ", name));

                if i == 0 {
                    state_calculations.push_str("\n\t\t\t\tlet field_value = state_id;");
                } else {
                    state_calculations.push_str(&format!(
                        "\n\t\t\t\tlet field_value = state_id.rem_euclid({});\
                        \n\t\t\t\tstate_id -= field_value;\
                        \n\t\t\t\tstate_id /= {};",
                        state.num_values, state.num_values
                    ));
                }

                match state.ty.as_str() {
                    "enum" => {
                        state_calculations.push_str(&format!(
                            "\n\t\t\t\tlet {}: {} = unsafe{{std::mem::transmute(field_value as u8)}};\n",
                            name, ty
                        ));
                    }
                    "int" => {
                        let values: Vec<i128> = state
                            .values
                            .as_ref()
                            .expect("No values for int block state")
                            .iter()
                            .map(|v| v.parse().expect("Invalid block state value: expected int"))
                            .collect();

                        let mut expected = values[0];
                        let mut standard = true;
                        for value in &values {
                            if value != &expected {
                                standard = false;
                                break;
                            }
                            expected += 1;
                        }

                        if standard && values[0] == 0 {
                            state_calculations.push_str(&format!(
                                "\n\t\t\t\tlet {}: {} = field_value as {};\n",
                                name, ty, ty
                            ));
                        } else if standard {
                            state_calculations.push_str(&format!(
                                "\n\t\t\t\tlet {}: {} = {} + field_value as {};\n",
                                name, ty, values[0], ty
                            ));
                        } else {
                            state_calculations.push_str(&format!(
                                "\n\t\t\t\tlet {}: {} = {:?}[field_value as usize];\n",
                                name, ty, values
                            ));
                        }
                    }
                    "bool" => {
                        state_calculations.push_str(&format!(
                            "\n\t\t\t\tlet {}: bool = field_value == 0;\n",
                            name
                        ));
                    }
                    other => panic!("Unknown {} type", other),
                }
            }

            state_id_match_arms.push_str(&format!(
                "
            {}..={} => {{
                state_id -= {};
                {}
                Some(BlockWithState::{}{{ {}}})
            }},",
                start, stop, start, state_calculations, name, fields
            ));
        }

        // Generate the code
        let code = format!(
            r#"//! Contains the [BlockWithState] enum to help with block state IDs.
            
use crate::*;

{enum_definitions}

/// Can be converted for free to [super::blocks::Block] which implements [useful methods](super::blocks::Block#implementations).
#[derive(Debug, Clone)]
#[repr(u32)]
pub enum BlockWithState {{
{variants}
}}

impl BlockWithState {{
    #[inline]
    pub fn from_state_id(mut state_id: u32) -> Option<BlockWithState> {{
        match state_id {{
{state_id_match_arms}
            _ => None,
        }}
    }}

    /// Returns the block id, **not the block state id**.
    #[inline]
    pub fn block_id(&self) -> u32 {{
        unsafe {{std::mem::transmute(std::mem::discriminant(self))}}
    }}
}}

impl From<super::blocks::Block> for BlockWithState {{
    #[inline]
    fn from(block: super::blocks::Block) -> BlockWithState {{
        BlockWithState::from_state_id(block.default_state_id()).unwrap() // TODO: unwrap unchecked
    }}
}}

impl<'a> MinecraftPacketPart<'a> for BlockWithState {{
    #[inline]
    fn serialize_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {{
        unimplemented!("Cannot serialize BlockWithState yet");
    }}

    #[inline]
    fn deserialize_minecraft_packet_part(input: &'a[u8]) -> Result<(Self, &'a[u8]), &'static str> {{
        let (id, input) = VarInt::deserialize_minecraft_packet_part(input)?;
        let id = std::cmp::max(id.0, 0) as u32;
        let block_with_state = BlockWithState::from_state_id(id).ok_or("No block corresponding to the specified block state ID.")?;
        Ok((block_with_state, input))
    }}
}}
"#,
            enum_definitions = enum_definitions_string,
            state_id_match_arms = state_id_match_arms,
            variants = variants,
        );

        File::create("src/ids/block_states.rs")
            .unwrap()
            .write_all(code.as_bytes())
            .unwrap()
    }
}

mod items {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Item {
        id: u32,
        display_name: String,
        #[serde(rename = "name")]
        text_id: String,
        stack_size: u8,
        durability: Option<u16>,
    }

    #[allow(clippy::explicit_counter_loop)]
    pub fn generate_item_enum(data: serde_json::Value) {
        let mut items: Vec<Item> = serde_json::from_value(data).expect("Invalid block data");
        items.sort_by_key(|item| item.id);

        // Look for missing items in the array
        let mut expected = 1;
        for item in &items {
            if item.id != expected {
                panic!("The item with id {} is missing.", expected)
            }
            expected += 1;
        }

        // Generate the variants of the Item enum
        let mut variants = String::new();
        for item in &items {
            let name = item
                .text_id
                .from_case(Case::Snake)
                .to_case(Case::UpperCamel);
            variants.push_str(&format!("\t{} = {},\n", name, item.id));
        }

        // Generate the code
        let code = format!(
            r#"use crate::*;

/// See [implementations](#implementations) for useful methods.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Item {{
{variants}
}}

impl Item {{
    #[inline]
    pub fn from_id(id: u32) -> Option<Item> {{
        if id < {max_value} {{
            Some(unsafe{{std::mem::transmute(id)}})
        }} else {{
            None
        }}
    }}

    #[inline]
    pub fn get_text_id(self) -> &'static str {{
        unsafe {{*TEXT_IDS.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn get_display_name(self) -> &'static str {{
        unsafe {{*DISPLAY_NAMES.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn get_max_stack_size(self) -> u8 {{
        unsafe {{*STACK_SIZES.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn get_durability(self) -> Option<u16> {{
        unsafe {{*DURABILITIES.get_unchecked((self as u32) as usize)}}
    }}
}}

impl<'a> MinecraftPacketPart<'a> for Item {{
    #[inline]
    fn serialize_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {{
        VarInt((self as u32) as i32).serialize_minecraft_packet_part(output)
    }}

    #[inline]
    fn deserialize_minecraft_packet_part(input: &'a[u8]) -> Result<(Self, &'a[u8]), &'static str> {{
        let (id, input) = VarInt::deserialize_minecraft_packet_part(input)?;
        let id = std::cmp::max(id.0, 0) as u32;
        let item = Item::from_id(id).ok_or("No item corresponding to the specified numeric ID.")?;
        Ok((item, input))
    }}
}}

const STACK_SIZES: [u8; {max_value}] = {max_stack_sizes:?};

const DURABILITIES: [Option<u16>; {max_value}] = {durabilities:?};

const DISPLAY_NAMES: [&str; {max_value}] = {display_names:?};

const TEXT_IDS: [&str; {max_value}] = {text_ids:?};
"#,
            variants = variants,
            max_value = expected - 1,
            max_stack_sizes = items.iter().map(|i| i.stack_size).collect::<Vec<_>>(),
            durabilities = items.iter().map(|i| i.durability).collect::<Vec<_>>(),
            display_names = items.iter().map(|i| &i.display_name).collect::<Vec<_>>(),
            text_ids = items.iter().map(|i| &i.text_id).collect::<Vec<_>>(),
        );

        File::create("src/ids/items.rs")
            .unwrap()
            .write_all(code.as_bytes())
            .unwrap()
    }
}

mod entities {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Entity {
        id: u32,
        #[serde(rename = "name")]
        text_id: String,
        display_name: String,
        width: f32,
        height: f32,
        #[serde(rename = "type")]
        category: String,
    }

    pub fn generate_entity_enum(data: serde_json::Value) {
        let mut entities: Vec<Entity> = serde_json::from_value(data).expect("Invalid entity data");
        entities.sort_by_key(|entity| entity.id);

        // Look for missing items in the array
        let mut expected = 0;
        for entity in &entities {
            if entity.id != expected {
                panic!("The entity with id {} is missing.", expected)
            }
            expected += 1;
        }

        // Generate the categories array
        let mut categories = String::new();
        categories.push('[');
        for entity in &entities {
            let variant_name = match entity.category.as_str() {
                "other" => "Other",
                "living" => "Living",
                "projectile" => "Projectile",
                "animal" => "Animal",
                "ambient" => "Ambient",
                "hostile" => "Hostile",
                "water_creature" => "WaterCreature",
                "mob" => "Mob",
                "passive" => "Passive",
                "player" => "Player",
                unknown_category => panic!("Unknown entity category {}", unknown_category),
            };
            categories.push_str("EntityCategory::");
            categories.push_str(variant_name);
            categories.push_str(", ");
        }
        categories.push(']');

        // Generate the variants of the Item enum
        let mut variants = String::new();
        for entity in &entities {
            let name = entity
                .text_id
                .from_case(Case::Snake)
                .to_case(Case::UpperCamel);
            variants.push_str(&format!("\t{} = {},\n", name, entity.id));
        }

        // Generate the code
        let code = format!(
            r#"use crate::*;

/// See [implementations](#implementations) for useful methods.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Entity {{
{variants}
}}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntityCategory {{
    Other,
    Living,
    Projectile,
    Animal,
    Ambient,
    Hostile,
    WaterCreature,
    Mob,
    Passive,
    Player,
}}

impl Entity {{
    #[inline]
    pub fn from_id(id: u32) -> Option<Entity> {{
        if id < {max_value} {{
            Some(unsafe{{std::mem::transmute(id)}})
        }} else {{
            None
        }}
    }}

    #[inline]
    pub fn get_text_id(self) -> &'static str {{
        unsafe {{*TEXT_IDS.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn get_display_name(self) -> &'static str {{
        unsafe {{*DISPLAY_NAMES.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn get_category(self) -> EntityCategory {{
        unsafe {{*CATEGORIES.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn get_height(self) -> f32 {{
        unsafe {{*HEIGHTS.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn get_width(self) -> f32 {{
        unsafe {{*WIDTHS.get_unchecked((self as u32) as usize)}}
    }}
}}

impl<'a> MinecraftPacketPart<'a> for Entity {{
    #[inline]
    fn serialize_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {{
        VarInt((self as u32) as i32).serialize_minecraft_packet_part(output)
    }}

    #[inline]
    fn deserialize_minecraft_packet_part(input: &'a[u8]) -> Result<(Self, &'a[u8]), &'static str> {{
        let (id, input) = VarInt::deserialize_minecraft_packet_part(input)?;
        let id = std::cmp::max(id.0, 0) as u32;
        let entity = Entity::from_id(id).ok_or("No entity corresponding to the specified numeric ID.")?;
        Ok((entity, input))
    }}
}}

const HEIGHTS: [f32; {max_value}] = {heights:?};

const WIDTHS: [f32; {max_value}] = {widths:?};

const DISPLAY_NAMES: [&str; {max_value}] = {display_names:?};

const TEXT_IDS: [&str; {max_value}] = {text_ids:?};

const CATEGORIES: [EntityCategory; {max_value}] = {categories};
"#,
            variants = variants,
            max_value = expected,
            heights = entities.iter().map(|e| e.height).collect::<Vec<_>>(),
            widths = entities.iter().map(|e| e.width).collect::<Vec<_>>(),
            display_names = entities.iter().map(|e| &e.display_name).collect::<Vec<_>>(),
            text_ids = entities.iter().map(|e| &e.text_id).collect::<Vec<_>>(),
            categories = categories,
        );

        File::create("src/ids/entities.rs")
            .unwrap()
            .write_all(code.as_bytes())
            .unwrap()
    }
}

fn main() {
    println!("cargo:rerun-if-changed=target/cache-file-location-{}.json", VERSION);

    let mut file_locations = get_data(
        "https://raw.githubusercontent.com/PrismarineJS/minecraft-data/master/data/dataPaths.json",
        &format!("target/cache-file-location-{}.json", VERSION),
    );
    let file_locations = file_locations.get_mut("pc").unwrap().take();
    let file_locations: HashMap<String, HashMap<String, String>> =
        serde_json::from_value(file_locations).unwrap();
    let file_locations = file_locations
        .get(VERSION)
        .expect("There is no generated data for this minecraft version yet");

    let blocks_url = format!(
        "https://github.com/PrismarineJS/minecraft-data/raw/master/data/{}/blocks.json",
        file_locations.get("blocks").unwrap()
    );
    let block_data = get_data(
        &blocks_url,
        &format!("target/cache-blocks-{}.json", VERSION),
    );
    blocks::generate_block_enum(block_data.clone());
    blocks::generate_block_with_state_enum(block_data);

    let items_url = format!(
        "https://github.com/PrismarineJS/minecraft-data/raw/master/data/{}/items.json",
        file_locations.get("items").unwrap()
    );
    let items_data = get_data(&items_url, &format!("target/cache-items-{}.json", VERSION));
    items::generate_item_enum(items_data);

    let entities_url = format!(
        "https://github.com/PrismarineJS/minecraft-data/raw/master/data/{}/entities.json",
        file_locations.get("entities").unwrap()
    );
    let entities_data = get_data(
        &entities_url,
        &format!("target/cache-entities-{}.json", VERSION),
    );
    entities::generate_entity_enum(entities_data);
}
