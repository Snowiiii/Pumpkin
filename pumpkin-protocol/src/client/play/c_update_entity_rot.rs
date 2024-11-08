use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:move_entity_rot")]
pub struct CUpdateEntityRot {
    entity_id: VarInt,
    yaw: u8,
    pitch: u8,
    on_ground: bool,
}

impl CUpdateEntityRot {
    pub fn new(entity_id: VarInt, yaw: u8, pitch: u8, on_ground: bool) -> Self {
        Self {
            entity_id,
            yaw,
            pitch,
            on_ground,
        }
    }
}
