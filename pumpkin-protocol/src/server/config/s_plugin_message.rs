use bytes::Bytes;
use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuf, ReadingError},
    Identifier, ServerPacket,
};

#[server_packet("config:custom_payload")]
pub struct SPluginMessage {
    pub channel: Identifier,
    pub data: bytes::Bytes,
}

impl ServerPacket for SPluginMessage {
    fn read(bytebuf: &mut Bytes) -> Result<Self, ReadingError> {
        Ok(Self {
            channel: bytebuf.try_get_string()?,
            data: bytebuf.split_to(bytebuf.len()),
        })
    }
}
