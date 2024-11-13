use pumpkin_macros::server_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[server_packet("play:move_player_status_only")]
pub struct SSetPlayerGround {
    pub on_ground: bool,
}
