use crate::block_state_registry::{BlockState, BlockStateRegistry};
use minecraft_protocol::components::blocks::BlockFace;
use minecraft_protocol::data::block_states::BlockWithState;
use minecraft_protocol::data::blocks::Block;
use minecraft_protocol::data::items::Item;

/// Database for mapping a block placement action to the final BlockWithState to be placed
pub struct BlockPlacementRegistry {
    block_id_to_placement_logic: Vec<PlacementLogic>,
}

impl BlockPlacementRegistry {
    /// When the ItemClickRegistry indicates that a BlockPlacement must happen, this function will tell
    /// which block will be placed.
    pub fn get_block(
        &self,
        block_state_registry: &BlockStateRegistry,
        held_item: Item,
        face: BlockFace,
        cursor_position_x: f32,
        cursor_position_y: f32,
        cursor_position_z: f32,
    ) -> BlockWithState {
        let block = self.item_id_to_block_id[held_item.id() as usize];
        let placement_logic = &self.block_id_to_placement_logic[block.id() as usize];

        let mut block_state = block_state_registry.get_default_block_state(block);

        match placement_logic {
            PlacementLogic::AlwaysUpright => block_state,
            PlacementLogic::Slab { .. } => {}
            PlacementLogic::Stair { .. } => {}
            PlacementLogic::Log { .. } => {}
            PlacementLogic::FacingPlayer { .. } => {}
            PlacementLogic::FacingPlayerHorizontal { .. } => {}
            PlacementLogic::Sign { .. } => {}
        }
    }
}

#[derive(Default)]
pub enum PlacementLogic {
    #[default]
    AlwaysUpright,
    Slab {
        /// Top = 0,
        /// Bottom = 1,
        /// Double = 2
        state_type: BlockState,
    },
    Stair {
        /// North = 0,
        /// South = 1,
        /// West = 2,
        /// East = 3
        state_facing: BlockState,
        /// Top = 0,
        /// Bottom = 1
        state_half: BlockState,
    },
    Log {
        /// X = 0,
        /// Y = 1,
        /// Z = 2
        state_axis: BlockState,
    },
    /// like droppers, pistons
    FacingPlayer {
        /// North = 0,
        /// East = 1,
        /// South = 2,
        /// West = 3,
        /// Up = 4,
        /// Down = 5
        state_facing: BlockState,
    },
    /// like furnaces
    FacingPlayerHorizontal {
        /// North = 0,
        /// South = 1,
        /// West = 2,
        /// East = 3
        state_facing: BlockState,
    },
    /// 16 possible values, always facing player
    Sign {
        /// "rotation", 0..=15
        state_rotation: BlockState,
    },
}
