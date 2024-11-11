use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("play:move_player_pos_rot")]
pub struct SPlayerPositionRotation {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
}
