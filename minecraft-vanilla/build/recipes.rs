use convert_case::{Case, Casing};
use minecraft_external::json::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

fn to_id_and_count(item: &CountedItem) -> (u32, u8) {
    match item {
        CountedItem::IDAndMetadataAndCount { .. } => panic!("Metadata not handled"),
        CountedItem::IDAndMetadata { .. } => panic!("Metadata not handled"),
        CountedItem::IDAndCount { id, count } => (*id, *count),
        CountedItem::ID(id) => (*id, 1),
    }
}

fn format_counted_item(item: &CountedItem, items: &[Item]) -> String {
    let (id, count) = to_id_and_count(item);
    let item_ident = item_id_to_item(id, items);
    format!(
        "CountedItem {{item: Item::{}, count: {}}}",
        item_ident, count
    )
}

fn format_count1_counted_item(item: &CountedItem, items: &[Item]) -> String {
    let (id, count) = to_id_and_count(item);
    assert_eq!(count, 1);
    let item_ident = item_id_to_item(id, items);
    format!("Item::{}", item_ident)
}

#[allow(dead_code)]
fn format_option_item(item: &Option<CountedItem>, items: &[Item]) -> String {
    match item {
        Some(item) => format!("Some({})", format_counted_item(item, items)),
        None => "None".to_string(),
    }
}

fn format_option_item_count1(item: &Option<CountedItem>, items: &[Item]) -> String {
    match item {
        Some(item) => format!("Some({})", format_count1_counted_item(item, items)),
        None => "None".to_string(),
    }
}

#[allow(dead_code)]
fn format_shape(shape: &Shape, i: &[Item]) -> String {
    match shape {
        Shape::ThreeByThree([[v1, v2, v3], [v4, v5, v6], [v7, v8, v9]]) => {
            format!(
                "Shape::ThreeByThree([[{}, {}, {}], [{}, {}, {}], [{}, {}, {}]])",
                format_option_item(v1, i),
                format_option_item(v2, i),
                format_option_item(v3, i),
                format_option_item(v4, i),
                format_option_item(v5, i),
                format_option_item(v6, i),
                format_option_item(v7, i),
                format_option_item(v8, i),
                format_option_item(v9, i)
            )
        }
        Shape::ThreeByTwo([[v1, v2, v3], [v4, v5, v6]]) => {
            format!(
                "Shape::ThreeByTwo([[{}, {}, {}], [{}, {}, {}]])",
                format_option_item(v1, i),
                format_option_item(v2, i),
                format_option_item(v3, i),
                format_option_item(v4, i),
                format_option_item(v5, i),
                format_option_item(v6, i)
            )
        }
        Shape::ThreeByOne([[v1, v2, v3]]) => {
            format!(
                "Shape::ThreeByOne([[{}, {}, {}]])",
                format_option_item(v1, i),
                format_option_item(v2, i),
                format_option_item(v3, i)
            )
        }
        Shape::TwoByThree([[v1, v2], [v3, v4], [v5, v6]]) => {
            format!(
                "Shape::TwoByThree([[{}, {}], [{}, {}], [{}, {}]])",
                format_option_item(v1, i),
                format_option_item(v2, i),
                format_option_item(v3, i),
                format_option_item(v4, i),
                format_option_item(v5, i),
                format_option_item(v6, i)
            )
        }
        Shape::TwoByTwo([[v1, v2], [v3, v4]]) => {
            format!(
                "Shape::TwoByTwo([[{}, {}], [{}, {}]])",
                format_option_item(v1, i),
                format_option_item(v2, i),
                format_option_item(v3, i),
                format_option_item(v4, i)
            )
        }
        Shape::TwoByOne([[v1, v2]]) => {
            format!(
                "Shape::TwoByOne([[{}, {}]])",
                format_option_item(v1, i),
                format_option_item(v2, i)
            )
        }
        Shape::OneByThree([[v1], [v2], [v3]]) => {
            format!(
                "Shape::OneByThree([[{}], [{}], [{}]])",
                format_option_item(v1, i),
                format_option_item(v2, i),
                format_option_item(v3, i)
            )
        }
        Shape::OneByTwo([[v1], [v2]]) => {
            format!(
                "Shape::OneByTwo([[{}], [{}]])",
                format_option_item(v1, i),
                format_option_item(v2, i)
            )
        }
        Shape::OneByOne([[v1]]) => {
            format!("Shape::OneByOne([[{}]])", format_option_item(v1, i))
        }
    }
}

fn format_count1_shape(shape: &Shape, i: &[Item]) -> String {
    match shape {
        Shape::ThreeByThree([[v1, v2, v3], [v4, v5, v6], [v7, v8, v9]]) => {
            format!(
                "Shape::ThreeByThree([[{}, {}, {}], [{}, {}, {}], [{}, {}, {}]])",
                format_option_item_count1(v1, i),
                format_option_item_count1(v2, i),
                format_option_item_count1(v3, i),
                format_option_item_count1(v4, i),
                format_option_item_count1(v5, i),
                format_option_item_count1(v6, i),
                format_option_item_count1(v7, i),
                format_option_item_count1(v8, i),
                format_option_item_count1(v9, i)
            )
        }
        Shape::ThreeByTwo([[v1, v2, v3], [v4, v5, v6]]) => {
            format!(
                "Shape::ThreeByTwo([[{}, {}, {}], [{}, {}, {}]])",
                format_option_item_count1(v1, i),
                format_option_item_count1(v2, i),
                format_option_item_count1(v3, i),
                format_option_item_count1(v4, i),
                format_option_item_count1(v5, i),
                format_option_item_count1(v6, i)
            )
        }
        Shape::ThreeByOne([[v1, v2, v3]]) => {
            format!(
                "Shape::ThreeByOne([[{}, {}, {}]])",
                format_option_item_count1(v1, i),
                format_option_item_count1(v2, i),
                format_option_item_count1(v3, i)
            )
        }
        Shape::TwoByThree([[v1, v2], [v3, v4], [v5, v6]]) => {
            format!(
                "Shape::TwoByThree([[{}, {}], [{}, {}], [{}, {}]])",
                format_option_item_count1(v1, i),
                format_option_item_count1(v2, i),
                format_option_item_count1(v3, i),
                format_option_item_count1(v4, i),
                format_option_item_count1(v5, i),
                format_option_item_count1(v6, i)
            )
        }
        Shape::TwoByTwo([[v1, v2], [v3, v4]]) => {
            format!(
                "Shape::TwoByTwo([[{}, {}], [{}, {}]])",
                format_option_item_count1(v1, i),
                format_option_item_count1(v2, i),
                format_option_item_count1(v3, i),
                format_option_item_count1(v4, i)
            )
        }
        Shape::TwoByOne([[v1, v2]]) => {
            format!(
                "Shape::TwoByOne([[{}, {}]])",
                format_option_item_count1(v1, i),
                format_option_item_count1(v2, i)
            )
        }
        Shape::OneByThree([[v1], [v2], [v3]]) => {
            format!(
                "Shape::OneByThree([[{}], [{}], [{}]])",
                format_option_item_count1(v1, i),
                format_option_item_count1(v2, i),
                format_option_item_count1(v3, i)
            )
        }
        Shape::OneByTwo([[v1], [v2]]) => {
            format!(
                "Shape::OneByTwo([[{}], [{}]])",
                format_option_item_count1(v1, i),
                format_option_item_count1(v2, i)
            )
        }
        Shape::OneByOne([[v1]]) => {
            format!("Shape::OneByOne([[{}]])", format_option_item_count1(v1, i))
        }
    }
}

fn item_id_to_item(id: u32, items: &[Item]) -> String {
    for item in items {
        if item.id == id {
            return item
                .internal_name
                .from_case(Case::Snake)
                .to_case(Case::UpperCamel);
        }
    }

    panic!("Item ID from recipe not found")
}

pub fn generate_recipes(
    item_recipes: HashMap<u32, Vec<Recipe>>,
    items: Vec<Item>,
    file: &mut File,
) {
    // Generate recipes
    let mut num_recipes = 0;
    let mut recipes_data = String::new();
    for recipes in item_recipes.values() {
        for recipe in recipes {
            match recipe {
                Recipe::ShapeLess {
                    result,
                    ingredients,
                } => {
                    let mut ingredients_string = String::new();
                    for ingredient in ingredients {
                        ingredients_string.push_str(&format_count1_counted_item(ingredient, &items));
                        ingredients_string.push_str(", ");
                    }

                    recipes_data.push_str(&format!(
                        "\tRecipe::ShapeLess {{ result: {}, ingredients: &[{}] }},\n",
                        format_counted_item(result, &items),
                        ingredients_string,
                    ));
                    num_recipes += 1;
                }
                Recipe::Shaped { result, in_shape } => {
                    recipes_data.push_str(&format!(
                        "\tRecipe::Shaped {{ result: {}, in_shape: {} }},\n",
                        format_counted_item(result, &items),
                        format_count1_shape(in_shape, &items),
                    ));
                    num_recipes += 1;
                }
                Recipe::DoubleShaped {
                    result,
                    in_shape,
                    out_shape,
                } => {
                    recipes_data.push_str(&format!(
                        "\tRecipe::DoubleShaped {{ result: {}, in_shape: {}, out_shape: {} }},\n",
                        format_counted_item(result, &items),
                        format_count1_shape(in_shape, &items),
                        format_count1_shape(out_shape, &items),
                    ));
                    num_recipes += 1;
                }
            }
        }
    }

    // Generate shortcuts
    let mut idx_in_array = 0;
    let mut shortcuts = Vec::new();
    for item_id in 0..items.len() {
        let vec_default = Vec::new();
        let recipes = item_recipes.get(&(item_id as u32)).unwrap_or(&vec_default);
        shortcuts.push((idx_in_array, idx_in_array + recipes.len()));
        idx_in_array += recipes.len();
    }

    #[allow(clippy::useless_format)]
    let code = format!(
        r#"//! All crafting recipes
use minecraft_protocol::data::recipes::{{CountedItem, Recipe}};
use crate::data::items::Item;

const RECIPES: [Recipe; {recipes_count}] = [
{recipes_data}
];

const SHORTCUTS: [(usize, usize); {item_count}] = {shortcuts:?};
"#,
        recipes_count = num_recipes,
        recipes_data = recipes_data,
        item_count = items.len(),
        shortcuts = shortcuts,
    );

    file.write_all(code.as_bytes()).unwrap()
}
