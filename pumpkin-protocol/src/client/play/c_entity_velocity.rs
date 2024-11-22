use pumpkin_core::math::vector3::Vector3;
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
    pub fn new(entity_id: &'a VarInt, velocity: Vector3<f64>) -> Self {
        Self {
            entity_id,
            velocity_x: (velocity.x.clamp(-3.9, 3.9) * 8000.0) as i16,
            velocity_y: (velocity.y.clamp(-3.9, 3.9) * 8000.0) as i16,
            velocity_z: (velocity.z.clamp(-3.9, 3.9) * 8000.0) as i16,
        }
    }
}
