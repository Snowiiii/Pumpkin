use bytes::Buf;
use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuf, ReadingError},
    codec::identifier::Identifier,
    ServerPacket,
};
const MAX_PAYLOAD_SIZE: usize = 1048576;

#[server_packet("config:custom_payload")]
pub struct SPluginMessage {
    pub channel: Identifier,
    pub data: bytes::Bytes,
}

impl ServerPacket for SPluginMessage {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError> {
        Ok(Self {
            channel: bytebuf.try_get_identifer()?,
            data: bytebuf.try_copy_to_bytes_len(bytebuf.remaining(), MAX_PAYLOAD_SIZE)?,
        })
    }
}
