use noise::Perlin;
use pumpkin_core::math::vector2::Vector2;

use crate::{
    biome::Biome,
    block::BlockId,
    coordinates::{BlockCoordinates, XZBlockCoordinates},
    world_gen::{
        generator::{BiomeGenerator, GeneratorInit, PerlinTerrainGenerator},
        generic_generator::GenericGenerator,
        Seed,
    },
};

pub type PlainsGenerator = GenericGenerator<PlainsBiomeGenerator, PlainsTerrainGenerator>;

pub(crate) struct PlainsBiomeGenerator {}

impl GeneratorInit for PlainsBiomeGenerator {
    fn new(_: Seed) -> Self {
        Self {}
    }
}

impl BiomeGenerator for PlainsBiomeGenerator {
    // TODO make generic over Biome and allow changing the Biome in the config.
    fn generate_biome(&self, _: XZBlockCoordinates) -> Biome {
        Biome::Plains
    }
}

pub(crate) struct PlainsTerrainGenerator {}

impl GeneratorInit for PlainsTerrainGenerator {
    fn new(_: Seed) -> Self {
        Self {}
    }
}

impl PerlinTerrainGenerator for PlainsTerrainGenerator {
    fn prepare_chunk(&self, _at: &Vector2<i32>, _perlin: &Perlin) {}
    // TODO allow specifying which blocks should be at which height in the config.
    fn generate_block(&self, at: BlockCoordinates, chunk_height: i16, _: Biome) -> BlockId {
        let begin_stone_height = chunk_height - 5;
        let begin_dirt_height = chunk_height - 1;

        let y = *at.y;
        if y == -64 {
            BlockId::from_id(79) // BEDROCK
        } else if y >= -63 && y <= begin_stone_height {
            return BlockId::from_id(1); // STONE
        } else if y >= begin_stone_height && y < begin_dirt_height {
            return BlockId::from_id(10); // DIRT;
        } else if y == chunk_height - 1 {
            return BlockId::from_id(9); // GRASS BLOCK
        } else {
            BlockId::AIR
        }
    }
}
