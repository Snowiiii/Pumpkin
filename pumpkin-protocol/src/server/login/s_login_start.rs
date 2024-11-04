use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuffer, DeserializerError},
    ServerPacket,
};

#[server_packet("login:hello")]
pub struct SLoginStart {
    pub name: String, // 16
    pub uuid: uuid::Uuid,
}

impl ServerPacket for SLoginStart {
    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError> {
        Ok(Self {
            name: bytebuf.get_string_len(16)?,
            uuid: bytebuf.get_uuid()?,
        })
    }
}
