use std::fs::File;
use std::io::Write;
use convert_case::{Case, Casing};
use super::*;
use minecraft_external::json::{Block, BlockState};

fn block_state_type_name(block_state: &BlockState, block_name: &str, competing_definitions: bool) -> String {
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

fn block_state_define_enum(block_state: &BlockState, block_name: &str, competing_definitions: bool) -> String {
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

#[allow(clippy::explicit_counter_loop)]
pub fn generate_block_enum(blocks: &Vec<Block>) {
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
                "\t\t\t{}..={} => Some(Block::{}),\n",
                start, stop, name
            ));
        } else {
            state_id_match_arms.push_str(&format!("\t\t\t{} => Some(Block::{}),\n", start, name));
        }
    }

    // Generate the code
    let code = format!(
        r#"use crate::*;

// THIS FILE IS GENERATED AUTOMATICALLY.
// See {this_file}.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockMaterial {{
{material_variants}
}}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Block {{
{variants}
}}

impl Block {{
    #[inline]
    pub fn from_id(id: u32) -> Option<Block> {{
        if id < {num_blocks} {{
            // SAFETY: Block has repr(u32) and it is a simple type
            Some(unsafe{{*(&raw const id).cast::<Block>()}})
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

impl From<super::block_states::BlockWithState> for Block {{
    #[inline]
    fn from(block_with_state: super::block_states::BlockWithState) -> Block {{
        // Every BlockWithState variant corresponds to the Block with the same id.
        // Because of this, the discriminants of associated blocks are equal.
        // See the comments on https://doc.rust-lang.org/stable/core/mem/fn.discriminant.html
		unsafe{{*(&raw const block_with_state).cast::<Block>()}}
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

const INTERNAL_NAMES: [&str; {num_blocks}] = {internal_names:?};
const DISPLAY_NAMES: [&str; {num_blocks}] = {display_names:?};
const MATERIALS: [[u8; 3]; {num_blocks}] = {block_materials:?};
"#,
        this_file = file!(),
        internal_names = blocks.iter().map(|b| &b.internal_name).collect::<Vec<_>>(),
        display_names = blocks.iter().map(|b| &b.display_name).collect::<Vec<_>>(),
    );

    File::create("src/ids/blocks.rs")
        .unwrap()
        .write_all(code.as_bytes())
        .unwrap()
}

#[allow(clippy::explicit_counter_loop)]
pub fn generate_block_with_state_enum(blocks: &Vec<Block>) {
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
        if !already_defined_enums
            .contains(&block_state_type_name(enum_definition, block_name, competing_definitions))
        {
            enum_definitions_string
                .push_str(&block_state_type_name(enum_definition, block_name, competing_definitions));
            enum_definitions_string.push('\n');
            enum_definitions_string.push('\n');

            already_defined_enums
                .push(block_state_type_name(enum_definition, block_name, competing_definitions));
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
            let competing_definitions =
                already_defined_enums.contains(&block_state_type_name(state, &block.internal_name, true));
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
                "\n\t\t\t{} => Some(BlockWithState::{}),",
                start, name
            ));
            state_id_rebuild_arms.push_str(&format!(
                "\n\t\t\tBlockWithState::{} => Some({}),",
                name, start
            ));
            continue;
        }

        let mut state_calculations = String::new();
        let mut fields = String::new();
        for (i, state) in block.states.iter().enumerate().rev() {
            let competing_definitions =
                already_defined_enums.contains(&block_state_type_name(state, &block.internal_name, true));
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
                Some(BlockWithState::{}{{ {}}})
            }},",
            start, stop, start, state_calculations, name, fields
        ));
        state_id_rebuild_arms.push_str(&format!(
            "
            BlockWithState::{}{{ {}}} => {{
                {}
                state_id += {};
                Some(state_id)
            }},",
            name, fields, state_reformation, start
        ));
    }

    // Generate the code
    let code = format!(
        r#"//! Contains the [BlockWithState] enum to help with block state IDs.
            
use crate::*;
use crate::ids::blocks::Block;

{enum_definitions}

/// Can be converted for free to [super::blocks::Block] which implements [useful methods](super::blocks::Block#implementations).
#[cfg_attr(test, derive(PartialEq))]
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

    #[inline]
    pub fn to_block(&self) -> Block {{
        // TODO: this is undefined behavior
		unsafe{{std::mem::transmute(std::mem::discriminant(self))}}
    }}

    /// Get the textual identifier of this block.
    #[inline]
    pub fn internal_name(self) -> &'static str {{
        self.to_block().internal_name()
    }}

    /// Get the english in-game name of this block.
    #[inline]
    pub fn display_name(self) -> &'static str {{
        self.to_block().display_name()
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

impl<'a> MinecraftPacketPart<'a> for BlockWithState {{
    #[inline]
    fn serialize_minecraft_packet_part(self, _output: &mut Vec<u8>) -> Result<(), &'static str> {{
        VarInt::from(self.block_state_id().unwrap_or(0)).serialize_minecraft_packet_part(_output)
    }}

    #[inline]
    fn deserialize_minecraft_packet_part(input: &'a[u8]) -> Result<(Self, &'a[u8]), &'static str> {{
        let (id, input) = VarInt::deserialize_minecraft_packet_part(input)?;
        let id = std::cmp::max(id.0, 0) as u32;
        let block_with_state = BlockWithState::from_state_id(id).ok_or("No block corresponding to the specified block state ID.")?;
        Ok((block_with_state, input))
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_block_states() {{
        for id in 0..={max_block_state_id} {{
            let block = BlockWithState::from_state_id(id).unwrap();
            let id_from_block = block.block_state_id().unwrap();
            assert_eq!(id, id_from_block);
        }}
    }}
}}
"#,
        enum_definitions = enum_definitions_string,
        state_id_match_arms = state_id_match_arms,
        state_id_rebuild_arms = state_id_rebuild_arms,
        variants = variants,
        max_block_state_id = blocks.last().unwrap().max_state_id
    );

    File::create("src/ids/block_states.rs")
        .unwrap()
        .write_all(code.as_bytes())
        .unwrap()
}
