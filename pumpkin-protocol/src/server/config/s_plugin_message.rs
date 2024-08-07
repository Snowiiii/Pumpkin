use pumpkin_macros::packet;

use crate::{
    bytebuf::{ByteBuffer, DeserializerError},
    Identifier, ServerPacket, VarInt,
};

#[packet(0x02)]
pub struct SPluginMessage {
    pub channel: Identifier,
    pub data: Vec<u8>,
}

impl ServerPacket for SPluginMessage {
    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError> {
        Ok(Self {
            channel: bytebuf.get_string().unwrap(),
            data: bytebuf.get_slice().to_vec(),
        })
    }
}
