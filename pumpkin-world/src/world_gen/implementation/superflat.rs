use pumpkin_core::math::vector2::Vector2;
use rle_vec::RleVec;

use crate::{
    block::block_state::BlockState,
    chunk::{BlockStorage, ChunkBlocks, ChunkData, SubChunkBlocks, SUBCHUNK_VOLUME},
    world_gen::WorldGenerator,
};

pub struct SuperflatGenerator {}

impl SuperflatGenerator {
    pub fn new() -> Self {
        Self {}
    }
}

impl WorldGenerator for SuperflatGenerator {
    fn generate_chunk(&self, position: Vector2<i32>) -> ChunkData {
        let mut blocks = ChunkBlocks::default();
        blocks.subchunks[5] = SubChunkBlocks::Multi({
            let mut vec = RleVec::new();
            vec.push_n(256, BlockState::new("minecraft:bedrock").unwrap().state_id);
            vec.push_n(512, BlockState::new("minecraft:dirt").unwrap().state_id);
            vec.push_n(
                256,
                BlockState::new("minecraft:grass_block").unwrap().state_id,
            );
            vec.push_n(SUBCHUNK_VOLUME - 1024, 0);
            BlockStorage::RleVec(vec)
        });
        ChunkData {
            position,
            blocks,
            heightmap: Default::default(),
        }
    }
}
