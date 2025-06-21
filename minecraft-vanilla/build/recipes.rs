use minecraft_external::json::*;
use minecraft_protocol::data;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use minecraft_protocol::data::recipes::RecipeRegistry;

fn to_counted_item(item: CountedItem) -> data::recipes::CountedItem {
    match item {
        CountedItem::IDAndMetadataAndCount { .. } => panic!("Metadata not handled"),
        CountedItem::IDAndMetadata { .. } => panic!("Metadata not handled"),
        CountedItem::IDAndCount { id, count } => data::recipes::CountedItem {
            item: data::items::Item::from_id(id),
            count,
        },
        CountedItem::ID(id) => data::recipes::CountedItem {
            item: data::items::Item::from_id(id),
            count: 1,
        },
    }
}

fn to_item(item: &CountedItem) -> data::items::Item {
    match item {
        CountedItem::IDAndMetadataAndCount { .. } => panic!("Metadata not handled"),
        CountedItem::IDAndMetadata { .. } => panic!("Metadata not handled"),
        CountedItem::IDAndCount { id, .. } | CountedItem::ID(id) => {
            data::items::Item::from_id(*id)
        }
    }
}

pub fn get_recipes_registry(
    item_recipes: HashMap<u32, Vec<Recipe>>
) -> RecipeRegistry {
    let mut out_recipes = Vec::new();

    // Generate recipes
    for (_, recipes) in item_recipes {
        for recipe in recipes {
            match recipe {
                Recipe::ShapeLess {
                    result,
                    ingredients,
                } => {
                    out_recipes.push(data::recipes::Recipe::ShapeLess {
                        result: to_counted_item(result),
                        ingredients: ingredients
                            .iter()
                            .map(|item| to_item(item))
                            .collect(),
                    });
                }
                Recipe::Shaped { result, in_shape } => {
                    out_recipes.push(data::recipes::Recipe::Shaped {
                        in_shape: transmute_shape(in_shape),
                        result: to_counted_item(result),
                    })
                }
                Recipe::DoubleShaped { .. } => {
                    panic!("Double shaped recipes are not supported")
                }
            }
        }
    }

    RecipeRegistry::build(out_recipes)
}

fn transmute_shape(shape: Shape) -> data::recipes::Shape {
    match shape {
        Shape::ThreeByThree(shape) => {
            data::recipes::Shape::ThreeByThree(transmute_shape_inner(shape))
        }
        Shape::ThreeByTwo(shape) => data::recipes::Shape::ThreeByTwo(transmute_shape_inner(shape)),
        Shape::ThreeByOne(shape) => data::recipes::Shape::ThreeByOne(transmute_shape_inner(shape)),
        Shape::TwoByThree(shape) => data::recipes::Shape::TwoByThree(transmute_shape_inner(shape)),
        Shape::TwoByTwo(shape) => data::recipes::Shape::TwoByTwo(transmute_shape_inner(shape)),
        Shape::TwoByOne(shape) => data::recipes::Shape::TwoByOne(transmute_shape_inner(shape)),
        Shape::OneByThree(shape) => data::recipes::Shape::OneByThree(transmute_shape_inner(shape)),
        Shape::OneByTwo(shape) => data::recipes::Shape::OneByTwo(transmute_shape_inner(shape)),
        Shape::OneByOne(shape) => data::recipes::Shape::OneByOne(transmute_shape_inner(shape)),
    }
}

fn transmute_shape_inner<const R: usize, const C: usize>(
    shape: [[Option<CountedItem>; C]; R],
) -> [[Option<data::items::Item>; C]; R] {
    shape.map(|row| row.map(|item| item.map(|i| to_item(&i))))
}
