use std::collections::HashMap;
use std::ops::Range;
use serde::{Deserialize, Serialize};
use crate::data::items::Item;

#[derive(Serialize, Deserialize)]
pub struct RecipeBook {
    lookup: Vec<Range<usize>>,
    recipes: Vec<Recipe>,
}

/// An [Item](crate::data::items::Item) associated with a count of this item
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CountedItem {
    pub item: Item,
    pub count: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Shape {
    ThreeByThree([[Option<Item>; 3]; 3]),
    ThreeByTwo([[Option<Item>; 3]; 2]),
    ThreeByOne([[Option<Item>; 3]; 1]),
    TwoByThree([[Option<Item>; 2]; 3]),
    TwoByTwo([[Option<Item>; 2]; 2]),
    TwoByOne([[Option<Item>; 2]; 1]),
    OneByThree([[Option<Item>; 1]; 3]),
    OneByTwo([[Option<Item>; 1]; 2]),
    OneByOne([[Option<Item>; 1]; 1]),
}

impl Shape {
    /// Returns the size of the shape.
    /// (width, height)
    pub const fn size(&self) -> (u8, u8) {
        match self {
            Shape::ThreeByThree(_) => (3, 3),
            Shape::ThreeByTwo(_) => (3, 2),
            Shape::ThreeByOne(_) => (3, 1),
            Shape::TwoByThree(_) => (2, 3),
            Shape::TwoByTwo(_) => (2, 2),
            Shape::TwoByOne(_) => (2, 1),
            Shape::OneByThree(_) => (1, 3),
            Shape::OneByTwo(_) => (1, 2),
            Shape::OneByOne(_) => (1, 1),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Recipe {
    Shaped {
        in_shape: Shape,
        result: CountedItem,
    },
    ShapeLess {
        ingredients: Vec<Item>,
        result: CountedItem,
    },
}

impl RecipeBook {
    pub fn build(recipes: Vec<Recipe>) -> Self {
        let mut item_recipes: HashMap<u32, Vec<Recipe>> = HashMap::new();

        let mut max_item_id: u32 = 0;
        for r in recipes {
            let item_id = r.result().item.id();
            max_item_id = u32::max(max_item_id, item_id);
            item_recipes.entry(item_id).or_insert_with(Vec::new).push(r);
        }

        let mut recipes = Vec::new();

        // Generate shortcuts
        let mut lookup = Vec::new();
        for item_id in 0..max_item_id {
            if let Some(recipe_list) = item_recipes.remove(&item_id) {
                let idx_in_array = recipes.len();
                lookup.push(idx_in_array .. idx_in_array + recipe_list.len());
                recipes.extend(recipe_list);
            }
        }

        RecipeBook { lookup, recipes }
    }

    /// Returns all the recipes for an item
    #[inline]
    pub fn get_recipes_for_item(&self, item: Item) -> &[Recipe] {
        let range = &self.lookup[item.id() as usize];
        &self.recipes[range.to_owned()]
    }
}

impl Recipe {
    #[inline]
    pub const fn result(&self) -> &CountedItem {
        match self {
            Recipe::Shaped { result, .. } => result,
            Recipe::ShapeLess { result, .. } => result,
        }
    }

    #[inline]
    pub const fn in_shape(&self) -> Option<&Shape> {
        match self {
            Recipe::Shaped { in_shape, .. } => Some(in_shape),
            Recipe::ShapeLess { .. } => None,
        }
    }

    #[inline]
    pub fn ingredients(&self) -> Option<&[Item]> {
        match self {
            Recipe::Shaped { .. } => None,
            Recipe::ShapeLess { ingredients, .. } => Some(ingredients),
        }
    }
}
