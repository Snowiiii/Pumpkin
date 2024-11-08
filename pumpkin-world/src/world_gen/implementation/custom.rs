use pumpkin_core::math::vector2::Vector2;

use crate::{
    block::BlockState,
    chunk::{ChunkBlocks, ChunkData},
    coordinates::Height,
    world_gen::{generator::GeneratorInit, WorldGenerator},
};

pub struct CustomGenerator {
    biom: String,
    layers: Vec<(u16, u16)>,
}

impl CustomGenerator {
    pub fn new(biom: String, layers: &Vec<(i16, String)>) -> Self {
        Self {
            biom,
            layers: layers
                .iter()
                .map(|(height, block_id)| {
                    (
                        Height(*height).get_absolute(),
                        BlockState::new(block_id).unwrap().state_id,
                    )
                })
                .collect(),
        }
    }
}

impl WorldGenerator for CustomGenerator {
    fn generate_chunk(&self, position: Vector2<i32>) -> ChunkData {
        let mut blocks = ChunkBlocks::default();
        ChunkData {
            position,
            ..Default::default()
        }
    }
}
