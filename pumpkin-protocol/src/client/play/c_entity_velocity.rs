use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x5A)]
pub struct CEntityVelocity<'a> {
    entitiy_id: &'a VarInt,
    velocity_x: i16,
    velocity_y: i16,
    velocity_z: i16,
}

impl<'a> CEntityVelocity<'a> {
    pub fn new(entitiy_id: &'a VarInt, velocity_x: f32, velocity_y: f32, velocity_z: f32) -> Self {
        Self {
            entitiy_id,
            velocity_x: (velocity_x.clamp(-3.9, 3.9) * 8000.0) as i16,
            velocity_y: (velocity_y.clamp(-3.9, 3.9) * 8000.0) as i16,
            velocity_z: (velocity_z.clamp(-3.9, 3.9) * 8000.0) as i16,
        }
    }
}
