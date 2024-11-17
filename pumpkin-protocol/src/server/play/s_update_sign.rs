use pumpkin_core::math::position::WorldPosition;
use pumpkin_macros::server_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[server_packet("play:sign_update")]
pub struct SUpdateSign {
    pub location: WorldPosition,
    pub is_front_text: bool,
    pub line_1: String,
    pub line_2: String,
    pub line_3: String,
    pub line_4: String,
}
