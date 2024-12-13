use bytes::Bytes;
use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuf, ReadingError},
    FixedBitSet, ServerPacket, VarInt,
};

// derive(Deserialize)]
#[server_packet("play:chat")]
pub struct SChatMessage {
    pub message: String,
    pub timestamp: i64,
    pub salt: i64,
    pub signature: Option<Bytes>,
    pub message_count: VarInt,
    pub acknowledged: FixedBitSet,
}

// TODO
impl ServerPacket for SChatMessage {
    fn read(bytebuf: &mut Bytes) -> Result<Self, ReadingError> {
        Ok(Self {
            message: bytebuf.try_get_string()?,
            timestamp: bytebuf.try_get_i64()?,
            salt: bytebuf.try_get_i64()?,
            signature: bytebuf.try_get_option(|v| v.try_copy_to_bytes(256))?,
            message_count: bytebuf.try_get_var_int()?,
            acknowledged: bytebuf.try_get_fixed_bitset(20)?,
        })
    }
}
