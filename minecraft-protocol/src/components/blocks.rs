use data::blocks::Block;

use crate::{*, nbt::NbtTag};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, MinecraftPacketPart)]
pub struct BlockEntity {
    /// The packed section coordinates are relative to the chunk they are in values 0-15 are valid.
    /// ```python
    /// packed_xz = ((blockX & 15) << 4) | (blockZ & 15) # encode
    /// x = packed_xz >> 4, z = packed_xz & 15 # decode
    /// ```
    packed_xz: u8,
    /// The height relative to the world
    y: i16,
    /// The type of block entity
    ty: VarInt,
    /// The block entity's data, without the X, Y, and Z values
    pub data: NbtTag,
}

#[cfg_attr(test, derive(PartialEq))]
#[minecraft_enum(VarInt)]
#[derive(Debug)]
pub enum PartialDiggingState {
    Started,
    Cancelled,
    Finished,
}

/// See [the wiki](https://wiki.vg/Protocol#Player_Digging)
#[derive(PartialEq)]
#[minecraft_enum(VarInt)]
#[derive(Debug)]
pub enum DiggingState {
    Started,
    Cancelled,
    Finished,
    DropItemStack,
    DropItem,
    ShootArrowOrFinishEating,
    SwapItemInHand,
}

#[cfg_attr(test, derive(PartialEq))]
#[minecraft_enum(u8)]
#[derive(Debug)]
pub enum BlockFace {
    Bottom,
    Top,
    North,
    South,
    West,
    East,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, MinecraftPacketPart)]
pub struct MultiBlockChange<'a> {
    /// Chunk section coordinate (encoded chunk x and z with each 22 bits, and section y with 20 bits, from left to right).
    ///
    /// Use [MultiBlockChange::decode_chunk_section_position] and [MultiBlockChange::encode_chunk_section_position] to work with it.
    pub chunk_section_position: u64,
    /// Each entry is composed of the block id, shifted right by 12, and the relative block position in the chunk section (4 bits for x, z, and y, from left to right).
    ///
    /// Use [MultiBlockChange::decode_block] and [MultiBlockChange::encode_block] to work with it.
    pub blocks: Array<'a, VarLong, VarInt>,
}

impl<'a> MultiBlockChange<'a> {
    /// Takes the position of the chunk (block coordinate divided by 16 and rounded down).
    pub fn encode_chunk_section_position(x: i32, y: i32, z: i32) -> Result<u64, &'static str> {
        let x = match x < 0 {
            true => (x + 2i32.pow(22)) as u64,
            false => x as u64,
        };
        let y = match y < 0 {
            true => (y + 2i32.pow(20)) as u64,
            false => y as u64,
        };
        let z = match z < 0 {
            true => (z + 2i32.pow(22)) as u64,
            false => z as u64,
        };

        if x > 0x3FFFFF || y > 0xFFFFF || z > 0x3FFFFF {
            return Err(
                "Unable to encode block: found a value out of range for the protocol types.",
            );
        }

        Ok((x & 0x3FFFFF) << 42 | (y & 0xFFFFF) | (z & 0x3FFFFF) << 20)
    }

    /// Returns the position of the chunk (block coordinate divided by 16 and rounded down).
    pub fn decode_chunk_section_position(chunk_section_position: u64) -> (i32, i32, i32) {
        let mut x = (chunk_section_position >> 42) as i32;
        let mut y = (chunk_section_position << 44 >> 44) as i32;
        let mut z = (chunk_section_position << 22 >> 42) as i32;

        if x > 2i32.pow(21) {
            x -= 2i32.pow(22);
        }
        if y > 2i32.pow(19) {
            y -= 2i32.pow(20);
        }
        if z > 2i32.pow(21) {
            z -= 2i32.pow(22);
        }

        (x, y, z)
    }

    /// Takes the position of the block relatively to the position of the chunk passed in `chunk_section_position` and the state id of a block.
    /// Use [Block::get_default_state_id](crate::data::blocks::Block::get_default_state_id) to get the state id corresponding to a [Block](crate::data::blocks::Block).
    ///
    /// ```ignore
    /// // get the relative X coordinate
    /// let chunk_x = (x / 16.0).floor();
    /// let relative_x = x - chunk_x * 16;
    /// ```
    pub fn encode_block(block: u32, x: u8, y: u8, z: u8) -> Result<u64, &'static str> {
        if x > 0xF || y > 0xF || z > 0xF {
            return Err(
                "Unable to encode block: found a value out of range for the protocol types.",
            );
        }

        Ok((block as u64) << 12 | ((x as u64) << 8 | (y as u64) << 4 | z as u64))
    }

    /// Returns the position of the block in the chunk at coordinates `chunk_section_position` and the state id of the block.
    /// Use [Block::from_state_id](crate::data::blocks::Block::from_state_id) to get the corresponding [Block](crate::data::blocks::Block).
    ///
    /// ```ignore
    /// // get the absolute X coordinate
    /// let x = chunk_x * 16 + relative_x;
    /// ```
    pub fn decode_block(block: u64) -> (u32, u8, u8, u8) {
        let decoded_block = (block >> 12) as u32;
        let x = (block << 52 >> 60) as u8;
        let y = (block << 60 >> 60) as u8;
        let z = (block << 56 >> 60) as u8;
        (decoded_block, x, y, z)
    }
}

impl BlockEntity {
    pub fn new(relative_x: u8, world_y: i32, relative_z: u8, block_type: Block, data: NbtTag) -> Self
    {
        BlockEntity {
            packed_xz : ((relative_x & 0b1111) << 4) | (relative_z & 0b1111),
            y: world_y as i16,
            ty: VarInt::from(block_type.id()),
            data,
        }
    }

    pub fn x(&self) -> i32 { (self.packed_xz >> 4) as i32 }

    pub fn y(&self) -> i32 { self.y as i32 }

    pub fn z(&self) -> i32 { (self.packed_xz & 0b1111) as i32 }

    pub fn get_block(&self) -> Block { Block::from_id(self.ty.0 as u32) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_section_position() {
        let position = (15, 7, 23);
        let encoded =
            MultiBlockChange::encode_chunk_section_position(position.0, position.1, position.2)
                .unwrap();
        let decoded = MultiBlockChange::decode_chunk_section_position(encoded);
        assert_eq!(position, decoded);

        let position = (-15, 7, 23);
        let encoded =
            MultiBlockChange::encode_chunk_section_position(position.0, position.1, position.2)
                .unwrap();
        let decoded = MultiBlockChange::decode_chunk_section_position(encoded);
        assert_eq!(position, decoded);

        let position = (0, 0, 0);
        let encoded =
            MultiBlockChange::encode_chunk_section_position(position.0, position.1, position.2)
                .unwrap();
        let decoded = MultiBlockChange::decode_chunk_section_position(encoded);
        assert_eq!(position, decoded);

        let position = (-1651, -65, -54412);
        let encoded =
            MultiBlockChange::encode_chunk_section_position(position.0, position.1, position.2)
                .unwrap();
        let decoded = MultiBlockChange::decode_chunk_section_position(encoded);
        assert_eq!(position, decoded);
    }
}
