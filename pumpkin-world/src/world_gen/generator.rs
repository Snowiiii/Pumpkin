use pumpkin_core::math::vector2::Vector2;
use static_assertions::assert_obj_safe;

use crate::biome::Biome;
use crate::block::BlockId;
use crate::chunk::ChunkData;
use crate::coordinates::{BlockCoordinates, XZBlockCoordinates};
use crate::world_gen::Seed;

pub trait GeneratorInit {
    fn new(seed: Seed) -> Self;
}

pub trait WorldGenerator: Sync + Send {
    #[allow(dead_code)]
    fn generate_chunk(&self, at: Vector2<i32>) -> ChunkData;
}
assert_obj_safe! {WorldGenerator}

pub(crate) trait BiomeGenerator: Sync + Send {
    fn generate_biome(&self, at: XZBlockCoordinates) -> Biome;
}

pub(crate) trait TerrainGenerator: Sync + Send {
    fn generate_block(&self, at: BlockCoordinates, biome: Biome) -> BlockId;
}
