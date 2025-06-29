use minecraft_protocol::data::blocks::Block;
use minecraft_protocol::data::entities::Entity;
use minecraft_protocol::data::items::Item;
use crate::block_placement_registry::PlacementLogic;

/// Database for mapping a player action to the appropriate event based on the held item and the targeted block.
struct ItemClickRegistry {
    item_effects: Vec<ItemClickEvent>
}

enum MouseButton {
    Left,
    Middle,
    Right
}

impl ItemClickRegistry {
    pub fn get_item_click_event(item: Item, mouse: MouseButton, target_block: Block) -> ItemClickEvent {
        todo!()
    }
}

enum ItemClickEvent {
    /// The targeted block initiates breaking
    BlockBreak,
    /// The item is transformed into a block and placed against the face of the targeted block
    /// Check with the BlockPlacementRegistry what block is placed.
    BlockPlacement { replace: bool },
    /// In contrast to BlockPlacement, this changes the item to generate a block.
    /// In vanilla, this covers buckets with fluids.
    /// Check with the BlockPlacementRegistry what block is placed.
    BlockPlacementWithItemTransformation {
        item_left: Item,
        replace: bool,
    },
    /// In contrast to BlockPlacementWithItemTransformation, this changes the metadata of the item to generate a block.
    /// In vanilla, this covers flint and steel (replace == false), and stripping oak (replace == true)
    /// Check with the BlockPlacementRegistry what block is placed.
    BlockPlacementWithItemMetadataChange {
        item_left: Item,
        replace: bool,
    },
    /// Spawn the given entity at the targeted block.
    /// For spawn eggs, and armor stand placement
    EntitySpawn {
        entity: Entity,
    },
    /// Like EntitySpawn events, but moving in the direction of player looking rather than placed at the targeted block
    EntityThrowEvent {
        entity: Entity,
        speed: f32,
    },
}