use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:set_entity_motion")]
pub struct CEntityVelocity<'a> {
    entity_id: &'a VarInt,
    velocity_x: i16,
    velocity_y: i16,
    velocity_z: i16,
}

impl<'a> CEntityVelocity<'a> {
    pub fn new(entity_id: &'a VarInt, velocity_x: f64, velocity_y: f64, velocity_z: f64) -> Self {
        Self {
            entity_id,
            velocity_x: (velocity_x.clamp(-3.9, 3.9) * 8000.0) as i16,
            velocity_y: (velocity_y.clamp(-3.9, 3.9) * 8000.0) as i16,
            velocity_z: (velocity_z.clamp(-3.9, 3.9) * 8000.0) as i16,
        }
    }
}
