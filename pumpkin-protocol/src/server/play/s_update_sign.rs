use pumpkin_core::math::position::WorldPosition;
use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuffer, DeserializerError},
    ServerPacket,
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
        Ok(Self {
            location: WorldPosition::from_i64(bytebuf.get_i64()?),
            is_front_text: bytebuf.get_bool()?,
            line_1: bytebuf.get_string_len(386)?,
            line_2: bytebuf.get_string_len(386)?,
            line_3: bytebuf.get_string_len(386)?,
            line_4: bytebuf.get_string_len(386)?,
        })
    }
}
