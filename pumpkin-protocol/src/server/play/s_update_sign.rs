use pumpkin_core::math::position::WorldPosition;
use pumpkin_core::math::vector3::Vector3;
use pumpkin_macros::server_packet;
use serde::Deserialize;

use crate::{
    bytebuf::{ByteBuffer, DeserializerError},
    FixedBitSet, ServerPacket, VarInt,
};

#[server_packet("play:sign_update")]
pub struct SUpdateSign {
    pub location: WorldPosition,
    pub is_front_text: bool,
    pub line_1: String,
    pub line_2: String,
    pub line_3: String,
    pub line_4: String,
}

impl ServerPacket for SUpdateSign {
    fn read(bytebuf: &mut ByteBuffer) -> Result<Self, DeserializerError> {
        let location = WorldPosition(Vector3::new(
            bytebuf.get_i32()?,
            bytebuf.get_i32()?,
            bytebuf.get_i32()?,
        ));
        Ok(Self {
            location,
            is_front_text: bytebuf.get_bool()?,
            line_1: bytebuf.get_string_len(386)?,
            line_2: bytebuf.get_string_len(386)?,
            line_3: bytebuf.get_string_len(386)?,
            line_4: bytebuf.get_string_len(386)?,
        })
    }
}
