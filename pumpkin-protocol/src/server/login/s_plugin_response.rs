use bytes::BytesMut;
use pumpkin_macros::packet;

use crate::{
    bytebuf::{ByteBuffer, DeserializerError},
    ServerPacket, VarInt,
};

#[packet(0x02)]
pub struct SLoginPluginResponse {
    pub message_id: VarInt,
    pub data: Option<BytesMut>,
}

impl ServerPacket for SLoginPluginResponse {
    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError> {
        Ok(Self {
            message_id: bytebuf.get_var_int()?,
            data: bytebuf.get_option(|v| Ok(v.get_slice()))?,
        })
    }
}
