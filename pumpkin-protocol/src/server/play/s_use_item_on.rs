use pumpkin_macros::packet;
use serde::Deserialize;

use crate::{position::WorldPosition, VarInt};

#[derive(Deserialize)]
#[packet(0x38)]
pub struct SUseItemOn {
    pub hand: VarInt,
    pub location: WorldPosition,
    pub face: VarInt,
    pub cursor_pos_x: f32,
    pub cursor_pos_y: f32,
    pub cursor_pos_z: f32,
    pub inside_block: bool,
    pub sequence: VarInt,
}
