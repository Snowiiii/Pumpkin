use pumpkin_core::math::vector2::Vector2;

use crate::{chunk::ChunkData, world_gen::WorldGenerator};

pub struct VoidGenerator {}

impl VoidGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

impl WorldGenerator for VoidGenerator {
    fn generate_chunk(&self, position: Vector2<i32>) -> ChunkData {
        ChunkData {
            position,
            ..Default::default()
        }
    }
}
