use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("play:move_player_rot")]
pub struct SPlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
}
