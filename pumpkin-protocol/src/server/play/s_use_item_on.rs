use pumpkin_core::math::position::WorldPosition;
use pumpkin_macros::server_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[server_packet("play:use_item_on")]
pub struct SUseItemOn {
    pub hand: VarInt,
    pub location: WorldPosition,
    pub face: VarInt,
    pub cursor_pos_x: f32,
    pub cursor_pos_y: f32,
    pub cursor_pos_z: f32,
    pub inside_block: bool,
    pub is_against_world_border: bool,
    pub sequence: VarInt,
}
