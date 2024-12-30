use crate::{
    bytebuf::{ByteBuf, ReadingError},
    ServerPacket, VarInt,
};
use bytes::{Buf, Bytes};
use pumpkin_macros::server_packet;

const MAX_PAYLOAD_SIZE: usize = 1048576;

#[server_packet("login:custom_query_answer")]
pub struct SLoginPluginResponse {
    pub message_id: VarInt,
    pub data: Option<Bytes>,
}

impl ServerPacket for SLoginPluginResponse {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError> {
        Ok(Self {
            message_id: bytebuf.try_get_var_int()?,
            data: bytebuf
                .try_get_option(|v| v.try_copy_to_bytes_len(v.remaining(), MAX_PAYLOAD_SIZE))?,
        })
    }
}
