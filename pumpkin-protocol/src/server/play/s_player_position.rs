use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("play:move_player_pos")]
pub struct SPlayerPosition {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub ground: bool,
}
