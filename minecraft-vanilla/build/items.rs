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
        r#"
// THIS FILE IS GENERATED AUTOMATICALLY.
// See {this_file}.

use minecraft_protocol::data::items::Item;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemId {{
{variants}
}}

impl ItemId {{
    #[inline]
    pub fn text_id(self) -> &'static str {{
        unsafe {{*TEXT_IDS.get_unchecked((self as u32) as usize)}}
    }}

    #[inline]
    pub fn display_name(self) -> &'static str {{
        unsafe {{*DISPLAY_NAMES.get_unchecked((self as u32) as usize)}}
    }}
}}

impl From<Item> for ItemId {{
	fn from(value: Item) -> Self {{
		unsafe {{ std::mem::transmute(value) }}
	}}
}}

impl From<ItemId> for Item {{
	fn from(value: ItemId) -> Self {{
		unsafe {{ std::mem::transmute(value) }}
	}}
}}

const DISPLAY_NAMES: [&str; {max_value}] = {display_names:?};

const TEXT_IDS: [&str; {max_value}] = {text_ids:?};
"#,
        this_file = file!(),
        variants = variants,
        max_value = expected,
        display_names = items.iter().map(|i| &i.display_name).collect::<Vec<_>>(),
        text_ids = items.iter().map(|i| &i.internal_name).collect::<Vec<_>>(),
    );

    file.write_all(code.as_bytes()).unwrap();
}
