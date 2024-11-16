use serde::Deserialize;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_macros::server_packet;

#[derive(Deserialize)]
#[server_packet("play:sign_update")]
pub struct SUpdateSign {
    pub location: WorldPosition,
    pub is_front_text: bool,
    pub line1: String,
    pub line2: String,
    pub line3: String,
    pub line4: String,
}