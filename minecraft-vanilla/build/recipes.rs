use convert_case::{Case, Casing};
use minecraft_external::json::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

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
                        ingredients_string.push_str(&ingredient.format_count1(&items));
                        ingredients_string.push_str(", ");
                    }

                    recipes_data.push_str(&format!(
                        "\tRecipe::ShapeLess {{ result: {}, ingredients: &[{}] }},\n",
                        result.format(&items),
                        ingredients_string,
                    ));
                    num_recipes += 1;
                }
                Recipe::Shaped { result, in_shape } => {
                    recipes_data.push_str(&format!(
                        "\tRecipe::Shaped {{ result: {}, in_shape: {} }},\n",
                        result.format(&items),
                        in_shape.format_count1(&items),
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
                        result.format(&items),
                        in_shape.format_count1(&items),
                        out_shape.format_count1(&items),
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

impl Recipe {{
    /// Returns all the recipes for an item
    #[inline]
    pub fn get_recipes_for_item(item: Item) -> &'static [Recipe] {{
        unsafe {{
            let (start, end) = SHORTCUTS.get_unchecked(item.id() as usize);
            RECIPES.get_unchecked(*start..*end)
        }}
    }}

    #[inline]
    pub const fn result(&self) -> &CountedItem {{
        match self {{
            Recipe::Shaped {{ result, .. }} => result,
            Recipe::ShapeLess {{ result, .. }} => result,
        }}
    }}
}}

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
