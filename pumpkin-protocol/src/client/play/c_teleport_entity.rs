use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x70)]
pub struct CTeleportEntitiy {
    entity_id: VarInt,
    x: f64,
    y: f64,
    z: f64,
    yaw: u8,
    pitch: u8,
    on_ground: bool,
}

impl CTeleportEntitiy {
    pub fn new(
        entity_id: VarInt,
        x: f64,
        y: f64,
        z: f64,
        yaw: u8,
        pitch: u8,
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            x,
            y,
            z,
            yaw,
            pitch,
            on_ground,
        }
    }
}
