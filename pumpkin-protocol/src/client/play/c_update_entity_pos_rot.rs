use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x2F)]
pub struct CUpdateEntityPosRot {
    entity_id: VarInt,
    delta_x: i16,
    delta_y: i16,
    delta_z: i16,
    yaw: u8,
    pitch: u8,
    on_ground: bool,
}

impl CUpdateEntityPosRot {
    pub fn new(
        entity_id: VarInt,
        delta_x: i16,
        delta_y: i16,
        delta_z: i16,
        yaw: u8,
        pitch: u8,
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            delta_x,
            delta_y,
            delta_z,
            yaw,
            pitch,
            on_ground,
        }
    }
}
