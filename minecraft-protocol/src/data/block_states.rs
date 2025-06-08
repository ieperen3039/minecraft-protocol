use crate::*;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Copy)]
pub struct BlockWithState(u32);

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