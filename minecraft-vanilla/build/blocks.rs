use convert_case::{Case, Casing};
use minecraft_external::json::{Block, BlockState};
use minecraft_game_logic::block_state_registry::BlockStateRegistry;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use minecraft_protocol::data::blocks as protocol;
use minecraft_protocol::data::block_states::BlockWithState;

fn block_state_type_name(
    block_state: &BlockState,
    block_name: &str,
    competing_definitions: bool,
) -> String {
    match block_state.state_type.as_str() {
        "int" => {
            let values: Vec<i128> = block_state
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
            true => format!(
                "{}_{}",
                block_name.split("_").last().unwrap_or(block_name),
                block_state.name
            ),
            false => block_state.name.to_string(),
        }
        .from_case(Case::Snake)
        .to_case(Case::UpperCamel),
        "bool" => String::from("bool"),
        _ => unimplemented!(),
    }
}

fn block_state_define_enum(
    block_state: &BlockState,
    block_name: &str,
    competing_definitions: bool,
) -> String {
    if block_state.state_type.as_str() != "enum" {
        panic!("Called defined enum on non-enum");
    }

    let mut variants = String::new();
    for (i, value) in block_state
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
#[cfg_attr(test, derive(PartialEq))]
pub enum {} {{{}
}}"#,
        block_state_type_name(block_state, block_name, competing_definitions),
        variants
    )
}

pub fn get_block_registry(blocks: &Vec<Block>) -> BlockStateRegistry {
    let mut registry = BlockStateRegistry::new();

    for block in blocks {
        let mut state_values = Vec::new();
        for s in &block.states {
            state_values.push(s.num_values as u32);
        }
        registry.add(protocol::Block::from_id(block.id), state_values, BlockWithState::from_id(block.default_state));
    }
    
    registry
}

#[allow(clippy::explicit_counter_loop)]
pub fn generate_block_enum(blocks: &Vec<Block>, file: &mut File) {
    // Look for missing blocks in the array
    let mut expected = 0;
    for block in blocks {
        if block.id != expected {
            panic!("The block with id {} is missing.", expected)
        }
        expected += 1;
    }
    let num_blocks = expected as usize;

    let mut block_materials = Vec::new();

    // Process a few fields
    let mut raw_materials: Vec<String> = Vec::new();
    raw_materials.push(String::from("None"));

    for block in blocks {
        let mut material_indices = Vec::new();
        if let Some(material_string) = &block.material {
            for mat in material_string.split(';') {
                let new_mat = mat
                    .replace("/", "_")
                    .from_case(Case::Snake)
                    .to_case(Case::UpperCamel);

                let index = raw_materials
                    .iter()
                    .position(|x| *x == new_mat)
                    .unwrap_or_else(|| {
                        raw_materials.push(new_mat);
                        raw_materials.len() - 1
                    });

                material_indices.push(index as u8);
            }
        }

        // we want an array of exactly 3 elements
        block_materials.push([
            material_indices.get(0).cloned().unwrap_or(0),
            material_indices.get(1).cloned().unwrap_or(0),
            material_indices.get(2).cloned().unwrap_or(0),
        ]);
        // check if there is any material with 4 elements
        assert_eq!(
            material_indices.get(3),
            None,
            "{:?} has materials {:?}",
            &block.display_name,
            &block.material
        );
    }

    // Generate the MaterialBlock
    let mut material_variants = String::new();
    for (index, material) in raw_materials.iter().enumerate() {
        material_variants.push_str(&format!("\t{material} = {index},\n",));
    }

    // Generate the variants of the Block enum
    let mut variants = String::new();
    for block in blocks {
        let name = block
            .internal_name
            .from_case(Case::Snake)
            .to_case(Case::UpperCamel);
        variants.push_str(&format!("\t{} = {},\n", name, block.id));
    }

    // Generate the `match` of state ids
    let mut state_id_match_arms = String::new();
    for block in blocks {
        let name = block
            .internal_name
            .from_case(Case::Snake)
            .to_case(Case::UpperCamel);
        let start = block.min_state_id;
        let stop = block.max_state_id;
        if start != stop {
            state_id_match_arms.push_str(&format!(
                "\t\t\t{}..={} => Some(BlockId::{}),\n",
                start, stop, name
            ));
        } else {
            state_id_match_arms.push_str(&format!("\t\t\t{} => Some(BlockId::{}),\n", start, name));
        }
    }

    // Generate the code
    let code = format!(
        r#"
// THIS FILE IS GENERATED AUTOMATICALLY.
// See {this_file}.

use minecraft_protocol::data::blocks::Block;
use minecraft_protocol::data::block_states::BlockWithState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockMaterial {{
{material_variants}
}}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockId {{
{variants}
}}

impl BlockId {{
    pub fn from_state_id(state_id: u32) -> Option<BlockId> {{
        match state_id {{
{state_id_match_arms}
            _ => None,
        }}
    }}

    #[inline]
    pub fn id(self) -> u32 {{
        self as u32
    }}

    /// Get the textual identifier of this block.
    #[inline]
    pub fn internal_name(self) -> &'static str {{
        unsafe {{*INTERNAL_NAMES.get_unchecked((self as u32) as usize)}}
    }}

    /// Get the english in-game name of this block.
    #[inline]
    pub fn display_name(self) -> &'static str {{
        unsafe {{*DISPLAY_NAMES.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn is_material(self, material: BlockMaterial) -> bool {{
		let materials = unsafe {{MATERIALS.get_unchecked((self as u32) as usize)}};
		materials.contains(&(material as u8))
    }}
}}

impl From<Block> for BlockId {{
	fn from(value: Block) -> Self {{
		assert!(value.id() < {num_blocks});
		unsafe {{ std::mem::transmute(value.id()) }}
	}}
}}

impl From<BlockId> for Block {{
	fn from(value: BlockId) -> Self {{
		Block::from_id(value as u32)
	}}
}}

impl From<super::block_states::BlockWithStateId> for BlockId {{
    #[inline]
    fn from(block_with_state: super::block_states::BlockWithStateId) -> BlockId {{
        // Every BlockWithState variant corresponds to the Block with the same id.
        // Because of this, the discriminants of associated blocks are equal.
        // See the comments on https://doc.rust-lang.org/stable/core/mem/fn.discriminant.html
		unsafe{{*(&raw const block_with_state).cast::<BlockId>()}}
    }}
}}

const INTERNAL_NAMES: [&str; {num_blocks}] = {internal_names:?};
const DISPLAY_NAMES: [&str; {num_blocks}] = {display_names:?};
const MATERIALS: [[u8; 3]; {num_blocks}] = {block_materials:?};
"#,
        this_file = file!(),
        internal_names = blocks.iter().map(|b| &b.internal_name).collect::<Vec<_>>(),
        display_names = blocks.iter().map(|b| &b.display_name).collect::<Vec<_>>(),
    );

    file.write_all(code.as_bytes()).unwrap()
}

#[allow(clippy::explicit_counter_loop)]
pub fn generate_block_with_state_enum(blocks: &Vec<Block>, file: &mut File) {
    // Generate the enum definitions
    let mut enum_definitions = Vec::new();
    let mut enum_definitions_string = String::new();
    let mut already_defined_enums = Vec::new();
    for block in blocks {
        for state in &block.states {
            if state.state_type.as_str() == "enum" {
                enum_definitions.push((&block.internal_name, state));
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
        if !already_defined_enums.contains(&block_state_type_name(
            enum_definition,
            block_name,
            competing_definitions,
        )) {
            enum_definitions_string.push_str(&block_state_define_enum(
                enum_definition,
                block_name,
                competing_definitions,
            ));
            enum_definitions_string.push('\n');
            enum_definitions_string.push('\n');

            already_defined_enums.push(block_state_type_name(
                enum_definition,
                block_name,
                competing_definitions,
            ));
        }
    }

    // Generate the variants of the Block enum
    let mut variants = String::new();
    for block in blocks {
        let name = block
            .internal_name
            .from_case(Case::Snake)
            .to_case(Case::UpperCamel);
        let mut fields = String::new();
        for state in &block.states {
            let name = match state.name.as_str() == "type" {
                true => "ty",
                false => state.name.as_str(),
            };
            let competing_definitions = already_defined_enums.contains(&block_state_type_name(
                state,
                &block.internal_name,
                true,
            ));
            let doc = if state.state_type == "int" {
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

                match standard {
                    true => format!(
                        "\t\t/// Valid if {} <= {} <= {}\n",
                        values[0],
                        name,
                        values.last().unwrap()
                    ),
                    false => format!("\t\t/// Valid if {} in {:?}\n", name, values),
                }
            } else {
                String::new()
            };
            fields.push_str(&format!(
                "{}\t\t{}: {},\n",
                doc,
                name,
                block_state_type_name(state, &block.internal_name, competing_definitions)
            ));
        }
        if fields.is_empty() {
            variants.push_str(&format!("\t{},\n", name));
        } else {
            variants.push_str(&format!("\t{} {{\n{}\t}},\n", name, fields));
        }
    }

    // Generate the `match` of state ids
    let mut state_id_match_arms = String::new();
    let mut state_id_rebuild_arms = String::new();
    for block in blocks {
        let name = block
            .internal_name
            .from_case(Case::Snake)
            .to_case(Case::UpperCamel);
        let start = block.min_state_id;
        let stop = block.max_state_id;

        if block.states.is_empty() {
            state_id_match_arms.push_str(&format!(
                "\n\t\t\t{} => Some(BlockWithStateId::{}),",
                start, name
            ));
            state_id_rebuild_arms.push_str(&format!(
                "\n\t\t\tBlockWithStateId::{} => Some({}),",
                name, start
            ));
            continue;
        }

        let mut state_calculations = String::new();
        let mut fields = String::new();
        for (i, state) in block.states.iter().enumerate().rev() {
            let competing_definitions = already_defined_enums.contains(&block_state_type_name(
                state,
                &block.internal_name,
                true,
            ));
            let ty = block_state_type_name(state, &block.internal_name, competing_definitions);
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

            match state.state_type.as_str() {
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

        let mut state_reformation = String::new();
        for (i, state) in block.states.iter().enumerate() {
            let name = match state.name.as_str() {
                "type" => "ty",
                _ => &state.name,
            };

            match state.state_type.as_str() {
                "enum" => {
                    state_reformation.push_str(&format!(
                        "\n\t\t\t\tlet field_value = (*{} as u8) as u32;",
                        name
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
                        state_reformation.push_str(&format!(
                            "\n\t\t\t\tif *{name} > {max} {{ return None }}\
                                \n\t\t\t\tlet field_value = *{name} as u32;",
                            name = name,
                            max = values.last().unwrap()
                        ));
                    } else if standard {
                        state_reformation.push_str(&format!(
                            "\n\t\t\t\tif *{name} < {min} || *{name} > {max} {{ return None }}\
                                \n\t\t\t\tlet field_value = ({name} - {min}) as u32;",
                            name = name,
                            min = values[0],
                            max = values.last().unwrap()
                        ));
                    } else {
                        state_reformation.push_str(&format!(
                            "\n\t\t\t\tlet field_value = {:?}.find({})?;",
                            values, name
                        ));
                    }
                }
                "bool" => {
                    state_reformation.push_str(&format!(
                        "\n\t\t\t\tlet field_value = if *{} {{0}} else {{1}};",
                        name
                    ));
                }
                other => panic!("Unknown {} type", other),
            }

            if i == 0 {
                state_reformation.push_str("\n\t\t\t\tlet mut state_id = field_value;\n");
            } else {
                state_reformation.push_str(&format!(
                    "\n\t\t\t\tstate_id *= {};\
                        \n\t\t\t\tstate_id += field_value;\n",
                    state.num_values
                ));
            }
        }

        state_id_match_arms.push_str(&format!(
            "
            {}..={} => {{
                state_id -= {};
                {}
                Some(BlockWithStateId::{}{{ {}}})
            }},",
            start, stop, start, state_calculations, name, fields
        ));
        state_id_rebuild_arms.push_str(&format!(
            "
            BlockWithStateId::{}{{ {}}} => {{
                {}
                state_id += {};
                Some(state_id)
            }},",
            name, fields, state_reformation, start
        ));
    }

    // Generate the code
    let code = format!(
        r#"
// THIS FILE IS GENERATED AUTOMATICALLY.
// See {this_file}.

use minecraft_protocol::{{packets::VarInt, MinecraftPacketPart}};
use minecraft_protocol::data::block_states::BlockWithState;
use crate::ids::blocks::BlockId;

{enum_definitions_string}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
#[repr(u32)]
pub enum BlockWithStateId {{
{variants}
}}

impl BlockWithStateId {{
    #[inline]
    pub fn from_id(mut state_id: u32) -> Option<BlockWithStateId> {{
        match state_id {{
{state_id_match_arms}
            _ => None,
        }}
    }}

    #[inline]
    pub fn to_block(&self) -> BlockId {{
        // TODO: this is undefined behavior
		unsafe{{std::mem::transmute(std::mem::discriminant(self))}}
    }}

    /// Get the textual identifier of this block.
    #[inline]
    pub fn internal_name(self) -> &'static str {{
        BlockId::from(self).internal_name()
    }}

    /// Get the english in-game name of this block.
    #[inline]
    pub fn display_name(self) -> &'static str {{
        BlockId::from(self).display_name()
    }}

    /// Returns the block state id.
    /// Returns None in case of error (invalid field value).
    #[inline]
    pub fn block_state_id(&self) -> Option<u32> {{
        match self {{
{state_id_rebuild_arms}
        }}
    }}
}}

impl From<BlockWithState> for BlockWithStateId {{
	fn from(value: BlockWithState) -> Self {{
		assert!(value.id() <= 24134);
		Self::from_id(value.id()).unwrap()
	}}
}}

impl From<BlockWithStateId> for BlockWithState {{
	fn from(value: BlockWithStateId) -> Self {{
		BlockWithState::from_id(value.block_state_id().unwrap())
	}}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_block_states() {{
        for id in 0..={max_block_state_id} {{
            let block = BlockWithStateId::from_id(id).unwrap();
            let id_from_block = block.block_state_id().unwrap();
            assert_eq!(id, id_from_block);
        }}
    }}
}}
"#,
        this_file = file!(),
        state_id_match_arms = state_id_match_arms,
        state_id_rebuild_arms = state_id_rebuild_arms,
        variants = variants,
        max_block_state_id = blocks.last().unwrap().max_state_id
    );

    file.write_all(code.as_bytes()).unwrap()
}
