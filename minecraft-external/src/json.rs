use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub id: u32,
    #[serde(rename = "name")]
    pub internal_name: String,
    pub display_name: String,
    pub hardness: f32,
    pub resistance: f32,
    pub diggable: bool,
    pub transparent: bool,
    pub filter_light: u8,
    pub emit_light: u8,
    pub default_state: u32,
    pub min_state_id: u32,
    pub max_state_id: u32,
    pub drops: Vec<u32>,
    pub material: Option<String>,
    #[serde(default)]
    pub harvest_tools: HashMap<u32, bool>,
    pub states: Vec<BlockState>,
    pub bounding_box: BoundingBox
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BoundingBox {
    Block,
    Empty,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BlockState {
    pub name: String,
    #[serde(rename = "type")]
    pub state_type: String,
    pub num_values: usize,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "camelCase")]
pub struct BlockDropMapping {
    #[serde(rename = "block")]
    pub block_internal_name: String,
    pub drops: Vec<BlockItemDrop>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: u32,
    pub display_name: String,
    #[serde(rename = "name")]
    pub internal_name: String,
    pub stack_size: u8,
    pub max_durability: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(rename_all = "camelCase")]
pub struct BlockItemDrop {
    /// The internal name of the item, without minecraft: prefix
    #[serde(rename = "item")]
    pub item_internal_name: String,
    /// The percent chance of the item drop to occur
    pub drop_chance: f64,
    /// The min/max of number of items in this item drop stack
    /// There are items with a value of -1
    pub stack_size_range: [Option<i32>; 2],
    /// The required age of the block for the item drop to occur
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_age: Option<i32>,
    /// If silk touch is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silk_touch: Option<bool>,
    /// If not having silk touch is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_silk_touch: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub id: u32,
    #[serde(rename = "name")]
    pub text_id: String,
    pub display_name: String,
    pub width: f32,
    pub height: f32,
    #[serde(rename = "type")]
    pub category: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Recipe {
    #[serde(rename_all = "camelCase")]
    DoubleShaped {
        result: CountedItem,
        in_shape: Shape,
        out_shape: Shape,
    },
    #[serde(rename_all = "camelCase")]
    Shaped { in_shape: Shape, result: CountedItem },
    #[serde(rename_all = "camelCase")]
    ShapeLess {
        result: CountedItem,
        ingredients: Vec<CountedItem>,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum CountedItem {
    IDAndMetadataAndCount { id: u32, metadata: u32, count: u8 },
    IDAndMetadata { id: u32, metadata: u32 },
    IDAndCount { id: u32, count: u8 },
    ID(u32),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Shape {
    ThreeByThree([[Option<CountedItem>; 3]; 3]),
    ThreeByTwo([[Option<CountedItem>; 3]; 2]),
    ThreeByOne([[Option<CountedItem>; 3]; 1]),
    TwoByThree([[Option<CountedItem>; 2]; 3]),
    TwoByTwo([[Option<CountedItem>; 2]; 2]),
    TwoByOne([[Option<CountedItem>; 2]; 1]),
    OneByThree([[Option<CountedItem>; 1]; 3]),
    OneByTwo([[Option<CountedItem>; 1]; 2]),
    OneByOne([[Option<CountedItem>; 1]; 1]),
}
