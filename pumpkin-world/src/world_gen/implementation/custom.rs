use pumpkin_core::math::vector2::Vector2;

use crate::{
    block::BlockState,
    chunk::{ChunkBlocks, ChunkData},
    coordinates::Height,
    world_gen::WorldGenerator,
};

pub struct CustomGenerator {
    biom: String,
    layers: Vec<(Height, u16)>,
}

impl CustomGenerator {
    pub fn new(biom: String, layers: &[(i16, String)]) -> Self {
        Self {
            biom,
            layers: layers
                .iter()
                .map(|(height, block_id)| {
                    (Height(*height), BlockState::new(block_id).unwrap().state_id)
                })
                .collect(),
        }
    }
}

impl WorldGenerator for CustomGenerator {
    fn generate_chunk(&self, position: Vector2<i32>) -> ChunkData {
        let mut blocks = ChunkBlocks::default();
        for layer in &self.layers {
            blocks.set_layer(layer.0, layer.1);
        }
        ChunkData {
            position,
            ..Default::default()
        }
    }
}
