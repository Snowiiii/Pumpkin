use pumpkin_macros::packet;
use serde::Serialize;

use crate::{uuid::UUID, VarInt};

#[derive(Serialize, Clone)]
#[packet(0x01)]
pub struct CSpawnEntity {
    entity_id: VarInt,
    entity_uuid: UUID,
    typ: VarInt,
    x: f64,
    y: f64,
    z: f64,
    pitch: u8,    // angle
    yaw: u8,      // angle
    head_yaw: u8, // angle
    data: VarInt,
    velocity_x: i16,
    velocity_y: i16,
    velocity_z: i16,
}

impl CSpawnEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entity_id: VarInt,
        entity_uuid: UUID,
        typ: VarInt,
        x: f64,
        y: f64,
        z: f64,
        pitch: f32,    // angle
        yaw: f32,      // angle
        head_yaw: f32, // angle
        data: VarInt,
        velocity_x: f32,
        velocity_y: f32,
        velocity_z: f32,
    ) -> Self {
        Self {
            entity_id,
            entity_uuid,
            typ,
            x,
            y,
            z,
            pitch: (pitch * 256.0 / 360.0).floor() as u8,
            yaw: (yaw * 256.0 / 360.0).floor() as u8,
            head_yaw: (head_yaw * 256.0 / 360.0).floor() as u8,
            data,
            velocity_x: (velocity_x.clamp(-3.9, 3.9) * 8000.0) as i16,
            velocity_y: (velocity_y.clamp(-3.9, 3.9) * 8000.0) as i16,
            velocity_z: (velocity_z.clamp(-3.9, 3.9) * 8000.0) as i16,
        }
    }
}
