use serde::{Deserialize, Serialize};
use crate::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Entity(u32);

impl Entity {
    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn from_id(id: u32) -> Self {
        Entity(id)
    }
}

impl<'a> MinecraftPacketPart<'a> for Entity {
    #[inline]
    fn serialize_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {
        VarInt::from(self.0).serialize_minecraft_packet_part(output)
    }

    #[inline]
    fn deserialize_minecraft_packet_part(
        input: &'a [u8],
    ) -> Result<(Self, &'a [u8]), &'static str> {
        let (id, input) = VarInt::deserialize_minecraft_packet_part(input)?;
        Ok((Entity(id.0 as u32), input))
    }
}
