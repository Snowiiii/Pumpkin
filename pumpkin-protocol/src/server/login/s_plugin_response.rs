use crate::{
    bytebuf::{ByteBuf, ReadingError},
    ServerPacket, VarInt,
};
use bytes::Bytes;
use pumpkin_macros::server_packet;

#[server_packet("login:custom_query_answer")]
pub struct SLoginPluginResponse {
    pub message_id: VarInt,
    pub data: Option<Bytes>,
}

impl ServerPacket for SLoginPluginResponse {
    fn read(bytebuf: &mut Bytes) -> Result<Self, ReadingError> {
        Ok(Self {
            message_id: bytebuf.try_get_var_int()?,
            data: bytebuf.try_get_option(|v| Ok(v.split_to(v.len())))?,
        })
    }
}
