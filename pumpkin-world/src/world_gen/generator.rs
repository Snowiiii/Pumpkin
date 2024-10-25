use noise::Perlin;
use pumpkin_core::math::vector2::Vector2;

use crate::biome::Biome;
use crate::block::block_state::BlockState;
use crate::chunk::{ChunkBlocks, ChunkData};
use crate::coordinates::{BlockCoordinates, ChunkRelativeBlockCoordinates, XZBlockCoordinates};
use crate::world_gen::Seed;

pub trait GeneratorInit {
    fn new(seed: Seed) -> Self;
}

pub trait WorldGenerator: Sync + Send {
    fn generate_chunk(&self, at: Vector2<i32>) -> ChunkData;
}

pub(crate) trait BiomeGenerator: Sync + Send {
    fn generate_biome(&self, at: XZBlockCoordinates) -> Biome;
}

#[expect(dead_code)]
pub(crate) trait TerrainGenerator: Sync + Send {
    fn prepare_chunk(&self, at: &Vector2<i32>);

    /// Is static
    fn generate_block(&self, at: BlockCoordinates, biome: Biome) -> BlockState;
}

pub(crate) trait PerlinTerrainGenerator: Sync + Send {
    fn height_variation(&self) -> f64 {
        4.0
    }

    fn prepare_chunk(&self, at: &Vector2<i32>, perlin: &Perlin);

    /// Dependens on the perlin noise height
    fn generate_block(
        &self,
        coordinates: ChunkRelativeBlockCoordinates,
        at: BlockCoordinates,
        blocks: &mut ChunkBlocks,
        chunk_height: i16,
        biome: Biome,
    );
}
