use pumpkin_core::math::vector3::Vector3;
use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("play:move_player_pos_rot")]
pub struct SPlayerPositionRotation {
    pub position: Vector3<f64>,
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
}
