use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct Item(u32);

impl Item {
    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn from_id(id: u32) -> Item {
        Item(id)
    }
}

impl<'a> MinecraftPacketPart<'a> for Item {
    #[inline]
    fn serialize_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {
        VarInt::from(self.0).serialize_minecraft_packet_part(output)
    }

    #[inline]
    fn deserialize_minecraft_packet_part(
        input: &'a [u8],
    ) -> Result<(Self, &'a [u8]), &'static str> {
        let (id, input) = VarInt::deserialize_minecraft_packet_part(input)?;
        Ok((Item(id.0 as u32), input))
    }
}
