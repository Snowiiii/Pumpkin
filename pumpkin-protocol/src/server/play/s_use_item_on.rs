use pumpkin_core::math::{position::WorldPosition, vector3::Vector3};
use pumpkin_macros::server_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[server_packet("play:use_item_on")]
pub struct SUseItemOn {
    pub hand: VarInt,
    pub location: WorldPosition,
    pub face: VarInt,
    pub cursor_pos: Vector3<f32>,
    pub inside_block: bool,
    pub is_against_world_border: bool,
    pub sequence: VarInt,
}
