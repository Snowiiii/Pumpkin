use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:level_particles")]
pub struct CParticle<'a> {
    /// If true, particle distance increases from 256 to 65536.
    long_distance: bool,
    x: f64,
    y: f64,
    z: f64,
    offset_x: f32,
    offset_y: f32,
    offset_z: f32,
    max_speed: f32,
    particle_count: i32,
    pariticle_id: VarInt,
    data: &'a [u8],
}

impl<'a> CParticle<'a> {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        long_distance: bool,
        x: f64,
        y: f64,
        z: f64,
        offset_x: f32,
        offset_y: f32,
        offset_z: f32,
        max_speed: f32,
        particle_count: i32,
        pariticle_id: VarInt,
        data: &'a [u8],
    ) -> Self {
        Self {
            long_distance,
            x,
            y,
            z,
            offset_x,
            offset_y,
            offset_z,
            max_speed,
            particle_count,
            pariticle_id,
            data,
        }
    }
}
