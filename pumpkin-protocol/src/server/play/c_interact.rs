use pumpkin_macros::packet;
use serde::Deserialize;

use crate::{ServerPacket, VarInt};

#[packet(0x16)]
pub struct SInteract {
    pub entity_id: VarInt,
    pub typ: VarInt,
    pub target_x: Option<f32>,
    // don't ask me why, adding more values does not work :c
}

// TODO
impl ServerPacket for SInteract {
    fn read(
        bytebuf: &mut crate::bytebuf::ByteBuffer,
    ) -> Result<Self, crate::bytebuf::DeserializerError> {
        Ok(Self {
            entity_id: bytebuf.get_var_int(),
            typ: bytebuf.get_var_int(),
            target_x: bytebuf.get_option(|v| v.get_f32()),
        })
    }
}
