use pumpkin_macros::packet;

use crate::{
    bytebuf::{ByteBuffer, DeserializerError},
    ServerPacket, VarInt,
};

#[packet(0x02)]
pub struct SLoginPluginResponse<'a> {
    pub message_id: VarInt,
    pub successful: bool,
    pub data: Option<&'a [u8]>,
}

impl<'a> ServerPacket for SLoginPluginResponse<'a> {
    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError> {
        Ok(Self {
            message_id: bytebuf.get_var_int(),
            successful: bytebuf.get_bool(),
            data: None, // TODO
        })
    }
}
