use pumpkin_core::math::vector3::Vector3;
use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("play:move_player_pos")]
pub struct SPlayerPosition {
    pub position: Vector3<f64>,
    pub ground: bool,
}
