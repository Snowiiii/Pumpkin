use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuffer, DeserializerError},
    Identifier, ServerPacket,
};

#[server_packet("configuration:custom_payload")]
pub struct SPluginMessage {
    pub channel: Identifier,
    pub data: Vec<u8>,
}

impl ServerPacket for SPluginMessage {
    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError> {
        Ok(Self {
            channel: bytebuf.get_string()?,
            data: bytebuf.get_slice().to_vec(),
        })
    }
}
