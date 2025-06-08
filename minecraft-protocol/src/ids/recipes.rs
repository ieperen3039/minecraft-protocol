//! All crafting recipes

use crate::ids::items::Item;

/// An [Item](crate::ids::items::Item) associated with a count of this item
#[derive(Debug, Clone)]
pub struct CountedItem {
    pub item: Item,
    pub count: u8,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Recipe {
    Shaped { in_shape: Shape, result: CountedItem },
    ShapeLess { ingredients: &'static [Item], result: CountedItem },
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
    pub const fn ingredients(&self) -> Option<&'static [Item]> {
        match self {
            Recipe::Shaped { .. } => None,
            Recipe::ShapeLess { ingredients, .. } => Some(ingredients),
        }
    }
}