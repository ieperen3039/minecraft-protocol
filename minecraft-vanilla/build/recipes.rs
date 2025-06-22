use minecraft_external::json::*;
use minecraft_protocol::data;
use minecraft_game_logic::recipes::RecipeRegistry;
use std::collections::HashMap;

fn to_counted_item(item: CountedItem) -> minecraft_game_logic::recipes::CountedItem {
    match item {
        CountedItem::IDAndMetadataAndCount { .. } => panic!("Metadata not handled"),
        CountedItem::IDAndMetadata { .. } => panic!("Metadata not handled"),
        CountedItem::IDAndCount { id, count } => minecraft_game_logic::recipes::CountedItem {
            item: data::items::Item::from_id(id),
            count,
        },
        CountedItem::ID(id) => minecraft_game_logic::recipes::CountedItem {
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
                    out_recipes.push(minecraft_game_logic::recipes::Recipe::ShapeLess {
                        result: to_counted_item(result),
                        ingredients: ingredients
                            .iter()
                            .map(|item| to_item(item))
                            .collect(),
                    });
                }
                Recipe::Shaped { result, in_shape } => {
                    out_recipes.push(minecraft_game_logic::recipes::Recipe::Shaped {
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

fn transmute_shape(shape: Shape) -> minecraft_game_logic::recipes::Shape {
    match shape {
        Shape::ThreeByThree(shape) => {
            minecraft_game_logic::recipes::Shape::ThreeByThree(transmute_shape_inner(shape))
        }
        Shape::ThreeByTwo(shape) => minecraft_game_logic::recipes::Shape::ThreeByTwo(transmute_shape_inner(shape)),
        Shape::ThreeByOne(shape) => minecraft_game_logic::recipes::Shape::ThreeByOne(transmute_shape_inner(shape)),
        Shape::TwoByThree(shape) => minecraft_game_logic::recipes::Shape::TwoByThree(transmute_shape_inner(shape)),
        Shape::TwoByTwo(shape) => minecraft_game_logic::recipes::Shape::TwoByTwo(transmute_shape_inner(shape)),
        Shape::TwoByOne(shape) => minecraft_game_logic::recipes::Shape::TwoByOne(transmute_shape_inner(shape)),
        Shape::OneByThree(shape) => minecraft_game_logic::recipes::Shape::OneByThree(transmute_shape_inner(shape)),
        Shape::OneByTwo(shape) => minecraft_game_logic::recipes::Shape::OneByTwo(transmute_shape_inner(shape)),
        Shape::OneByOne(shape) => minecraft_game_logic::recipes::Shape::OneByOne(transmute_shape_inner(shape)),
    }
}

fn transmute_shape_inner<const R: usize, const C: usize>(
    shape: [[Option<CountedItem>; C]; R],
) -> [[Option<data::items::Item>; C]; R] {
    shape.map(|row| row.map(|item| item.map(|i| to_item(&i))))
}
