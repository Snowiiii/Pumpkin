use pumpkin_core::math::vector3::Vector3;
use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:level_particles")]
pub struct CParticle<'a> {
    /// If true, particle distance increases from 256 to 65536.
    long_distance: bool,
    position: Vector3<f64>,
    offset: Vector3<f64>,
    max_speed: f32,
    particle_count: i32,
    pariticle_id: VarInt,
    data: &'a [u8],
}

impl<'a> CParticle<'a> {
    pub fn new(
        long_distance: bool,
        position: Vector3<f64>,
        offset: Vector3<f64>,
        max_speed: f32,
        particle_count: i32,
        pariticle_id: VarInt,
        data: &'a [u8],
    ) -> Self {
        Self {
            long_distance,
            position,
            offset,
            max_speed,
            particle_count,
            pariticle_id,
            data,
        }
    }
}
