use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct BlockWithState(u32);

impl BlockWithState {
    #[inline]
    pub fn id(&self) -> u32 {
        self.0
    }

    #[inline]
    pub fn from_id(id: u32) -> BlockWithState {
        BlockWithState(id)
    }
}

impl Default for BlockWithState {
    fn default() -> Self {
        BlockWithState(0)
    }
}

impl<'a> MinecraftPacketPart<'a> for BlockWithState {
    #[inline]
    fn serialize_minecraft_packet_part(self, _output: &mut Vec<u8>) -> Result<(), &'static str> {
        VarInt::from(self.0).serialize_minecraft_packet_part(_output)
    }

    #[inline]
    fn deserialize_minecraft_packet_part(input: &'a[u8]) -> Result<(Self, &'a[u8]), &'static str> {
        let (id, input) = VarInt::deserialize_minecraft_packet_part(input)?;
        Ok((BlockWithState(id.0 as u32), input))
    }
}