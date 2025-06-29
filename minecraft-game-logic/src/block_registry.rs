use std::collections::HashMap;
use crate::tool_type::ToolType;
use std::ops::Range;
use minecraft_protocol::data::blocks::Block;

pub struct BlockRegistry {
    /// Per mod prefix a list of indices in the `blocks` table.
    /// Vanilla blocks may be found under "minecraft"
    prefixes: HashMap<String, Range<usize>>,
    blocks: Vec<BlockDataEntry>,
}

pub struct BlockDataEntry {
    /// The string identifier of the block, without mod prefix.
    /// This is different from the displayed name, and always the same for a particular block
    pub internal_name: String,
    /// If `None`, this block cannot be mined
    /// If `Some`, the value is always greater than 0.0
    /// see https://minecraft.fandom.com/wiki/Breaking
    pub hardness: Option<f32>,
    /// Tools in this list receive a bonus to the break time for the block
    pub appropriate_tools: Vec<ToolType>,
    /// Resistance to explosions.
    /// See https://minecraft.fandom.com/wiki/Explosion#Blast_resistance
    pub blast_resistance: f32,
    /// Whether light passes through the block.
    /// Doesn't mean it is actually see-through.
    /// See https://minecraft.fandom.com/wiki/Opacity
    pub is_transparent: bool,
    /// Whether redstone signals passes through the block.
    /// Seems to be linked to transparency on the wiki.
    /// See https://minecraft.fandom.com/wiki/Conductivity
    pub is_conductive: bool,
    /// Whether players and mobs can move through this block.
    /// Does not mean mobs can spawn on it; this is defined on a per-entity basis
    /// See https://minecraft.fandom.com/wiki/Solid_block
    pub is_solid: bool,
    /// See https://minecraft.fandom.com/wiki/Fluid
    pub is_liquid: bool,
    /// See https://minecraft.fandom.com/wiki/Fire
    pub is_flammable: bool,
    /// When a player clicks on this block with a block-item, this block is replaced with the placed block,
    /// rather than the block being placed against this.
    /// Examples are grasses and liquids.
    pub is_replaceable: bool,
    /// Light-filtering blocks decrease skylight by the given number of levels.
    /// See https://minecraft.fandom.com/wiki/Light#Light-filtering_blocks
    pub filter_light: u8,
    /// Light-emitting blocks spread light around themselves
    /// See https://minecraft.fandom.com/wiki/Light#Light-emitting_blocks
    pub emit_light: u8,
    /// Color of this block on a minimap.
    pub map_color: [u8; 3],
    /// Interaction when pushed by a piston
    pub piston_behavior: PistonBehaviour,
}

pub enum PistonBehaviour {
    Push,
    Break,
    Block,
}

pub enum BlockMaterial {
    None = 0,
    Default = 1,
    MineablePickaxe = 2,
    MineableShovel = 3,
    MineableAxe = 4,
    Plant = 5,
    Leaves = 6,
    MineableHoe = 7,
    Coweb = 8,
    Wool = 9,
    Gourd = 10,
    VineOrGlowLichen = 11,
}

impl BlockRegistry {
    pub fn get_block_data(&self, block: Block) -> &BlockDataEntry {
        &self.blocks[block.id() as usize]
    }
    
    pub fn get_block_data_by_name(&self, qualified_internal_name: &str) -> Option<&BlockDataEntry> {
        let mut name_parts = qualified_internal_name.split(":");
        let prefix = name_parts.next().expect("empty name");
        let internal_name = name_parts.next().expect("name was unqualified");
        
        let block_range = self.prefixes[prefix].clone();
        self.blocks[block_range].iter().find(|b| b.internal_name == internal_name)
    }
}