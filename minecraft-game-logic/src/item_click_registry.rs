use crate::block_registry::BlockRegistry;
use minecraft_protocol::data::blocks::Block;
use minecraft_protocol::data::entities::Entity;
use minecraft_protocol::data::items::Item;
use serde::{Deserialize, Serialize};

/// Database for mapping a player action to the appropriate event based on the held item and the targeted block.
/// ## Notes
/// The clicked block is not checked for interactions. One should first check if the clicked block is interactable,
/// and only if it is not, query the ItemClickRegistry
///
/// When a waterlog-able block is clicked with a water bucket, a BlockPlacement event is returned
/// with the targeted block as the base block, and `replace == true`.
/// In this case, the 'waterlogged' state of the original block must be set to 1.
#[derive(Serialize, Deserialize, Debug)]
struct ItemClickRegistry {
    /// maps a given item id to the base event that happens upon using the secondary mouse button
    base_event: Vec<ItemClickEventInternal>,
}

#[derive(Serialize, Deserialize, Debug)]
enum ItemClickEventInternal {
    /// generates some variant of a [BlockPlacement](ItemClickEvent::BlockPlacement) event based on the targeted block
    BlockPlacement {
        base_block: Block,
        change: ItemChange,
    },
    /// always generates the same event with the same parameters
    Wrapped(ItemClickEvent),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ItemClickEvent {
    /// Nothing happens.
    /// Clicking into the air with a block is the primary example.
    Nothing,
    /// Something out of the scope of this library is supposed to happen.
    /// The application may look into special cases after receiving this result.
    /// Vanilla examples include charging a bow, opening a map
    Something,
    /// A block is generated in the world, and the item is changed according to `ItemChange`.
    /// The `base_block` must be translated to the appropriate BlockWithState before it can be added to the world.
    BlockPlacement {
        base_block: Block,
        change: ItemChange,
        replace: bool,
    },
    /// the item is eaten over some period of time
    Eat {
        food_points: f32,
        saturation: f32,
        change: ItemChange,
    },
    /// Spawn the given entity at the targeted block.
    /// Intended for spawn eggs, and armor stand placement.
    EntitySpawn { entity: Entity },
    /// Like EntitySpawn events, but moving in the direction of player looking rather than placed at the targeted block.
    /// Intended for thrown items like ender pearls.
    // TODO intended for ender pearls, but no way of specifying how metadata is set
    EntityThrow { entity: Entity, speed: f32 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ItemChange {
    /// The item is removed from the inventory
    Consumed,
    /// The item is transformed into another item
    /// In vanilla, this covers buckets with fluids.
    Transformed { into: Item },
    /// The item is damaged.
    /// In vanilla, this covers using flint and steel.
    Damaged { quantity: u32 },
}

impl ItemClickRegistry {
    pub fn get_item_click_event(
        &self,
        block_registry: &BlockRegistry,
        item: Item,
        target_block: Option<Block>,
    ) -> ItemClickEvent {
        let event = &self.base_event[item.id() as usize];
        match event {
            ItemClickEventInternal::BlockPlacement { base_block, change } => {
                if let Some(target_block) = target_block {
                    let target_is_replaceable =
                        block_registry.get_block_data(target_block).is_replaceable;

                    ItemClickEvent::BlockPlacement {
                        base_block: base_block.clone(),
                        change: change.clone(),
                        replace: target_is_replaceable,
                    }
                } else {
                    ItemClickEvent::Nothing
                }
            }
            ItemClickEventInternal::Wrapped(event) => event.clone(),
        }
    }
}
