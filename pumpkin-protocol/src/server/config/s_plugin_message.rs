use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuffer, DeserializerError},
    Identifier, ServerPacket,
};

#[server_packet("config:custom_payload")]
pub struct SPluginMessage {
    pub channel: Identifier,
    pub data: bytes::BytesMut,
}

impl ServerPacket for SPluginMessage {
    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError> {
        Ok(Self {
            channel: bytebuf.get_string()?,
            data: bytebuf.get_slice(),
        })
    }
}
