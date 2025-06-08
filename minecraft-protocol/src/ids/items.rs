use crate::*;

// We do not implement Copy, to reduce the chance of item duplication
#[derive(Debug, Clone, PartialEq)]
pub struct Item(u32);

impl<'a> MinecraftPacketPart<'a> for Item {
	#[inline]
	fn serialize_minecraft_packet_part(self, output: &mut Vec<u8>) -> Result<(), &'static str> {
		VarInt::from(self.0).serialize_minecraft_packet_part(output)
	}

	#[inline]
	fn deserialize_minecraft_packet_part(input: &'a[u8]) -> Result<(Self, &'a[u8]), &'static str> {
		let (id, input) = VarInt::deserialize_minecraft_packet_part(input)?;
		Ok((Item(id.0 as u32), input))
	}
}