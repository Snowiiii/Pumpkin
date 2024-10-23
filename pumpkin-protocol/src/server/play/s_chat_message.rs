use bytes::Bytes;

use crate::{
    bytebuf::{ByteBuffer, DeserializerError},
    FixedBitSet, ServerPacket, VarInt,
};

// derive(Deserialize)]
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
    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError> {
        Ok(Self {
            message: bytebuf.get_string()?,
            timestamp: bytebuf.get_i64()?,
            salt: bytebuf.get_i64()?,
            signature: bytebuf.get_option(|v| v.copy_to_bytes(256))?,
            message_count: bytebuf.get_var_int()?,
            acknowledged: bytebuf.get_fixed_bitset(20)?,
        })
    }
}
