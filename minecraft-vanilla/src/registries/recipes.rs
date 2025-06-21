use minecraft_protocol::data::recipes::RecipeRegistry;

pub fn get_recipes() -> RecipeRegistry {
    let (result, _) = bincode::serde::decode_from_slice(
        include_bytes!("../../data/recipes.bin"),
        bincode::config::standard(),
    )
    .expect("Failed to decode data/recipes.bin");

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use minecraft_protocol::data::items::Item;
    use minecraft_protocol::data::recipes::Shape;
    use crate::ids::items::ItemId;

    #[test]
    fn test_get_recipes() {
        let book = get_recipes();
		let got = book.get_recipes_for_item(Item::from(ItemId::OakButton)); // ItemId::OakButton

        assert_eq!(got.len(), 1, "{got:?}");
        assert_eq!(got[0].result().item, Item::from(ItemId::OakButton));
        assert_eq!(got[0].result().count, 1);

        match got[0].in_shape() {
            Some(Shape::OneByOne(shape)) => {
                assert_eq!(shape[0][0], Some(Item::from(ItemId::OakPlanks)));
            }
            None => {
                assert_eq!(got[0].ingredients(), Some(&[Item::from(ItemId::OakPlanks)][..]));
            }
            _ => panic!("Unexpected shape"),
        }
    }
}
