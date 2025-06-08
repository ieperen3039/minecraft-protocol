use convert_case::{Case, Casing};
use minecraft_external::json::Item;
use std::fs::File;
use std::io::Write;

#[allow(clippy::explicit_counter_loop)]
pub fn generate_item_enum(items: &Vec<Item>, file: &mut File) {
    // Look for missing items in the array
    let mut expected = 0;
    for item in items {
        if item.id != expected {
            panic!("The item with id {} is missing.", expected)
        }
        expected += 1;
    }

    // Generate the variants of the Item enum
    let mut variants = String::new();
    for item in items {
        let name = item
            .internal_name
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
            // SAFETY: Item has repr(u32) and it is a simple type
            Some(unsafe{{*(&raw const id).cast::<Item>()}})
        }} else {{
            None
        }}
    }}

    #[inline]
    pub fn text_id(self) -> &'static str {{
        unsafe {{*TEXT_IDS.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn display_name(self) -> &'static str {{
        unsafe {{*DISPLAY_NAMES.get_unchecked((self as u32) as usize)}}
    }}
}}

impl<'a> MinecraftPacketPart<'a> for Item {{
    #[inline]
    fn serialize_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {{
        VarInt(self as i32).serialize_minecraft_packet_part(output)
    }}

    #[inline]
    fn deserialize_minecraft_packet_part(input: &'a[u8]) -> Result<(Self, &'a[u8]), &'static str> {{
        let (id, input) = VarInt::deserialize_minecraft_packet_part(input)?;
        let id = std::cmp::max(id.0, 0) as u32;
        let item = Item::from_id(id).ok_or("No item corresponding to the specified numeric ID.")?;
        Ok((item, input))
    }}
}}

const DISPLAY_NAMES: [&str; {max_value}] = {display_names:?};

const TEXT_IDS: [&str; {max_value}] = {text_ids:?};
"#,
        variants = variants,
        max_value = expected,
        display_names = items.iter().map(|i| &i.display_name).collect::<Vec<_>>(),
        text_ids = items.iter().map(|i| &i.internal_name).collect::<Vec<_>>(),
    );

    file.write_all(code.as_bytes()).unwrap();
}
