use bytes::Bytes;
use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuf, ReadingError},
    ServerPacket,
};

#[server_packet("login:hello")]
pub struct SLoginStart {
    pub name: String, // 16
    pub uuid: uuid::Uuid,
}

impl ServerPacket for SLoginStart {
    fn read(bytebuf: &mut Bytes) -> Result<Self, ReadingError> {
        Ok(Self {
            name: bytebuf.try_get_string_len(16)?,
            uuid: bytebuf.try_get_uuid()?,
        })
    }
}
