use noise::Perlin;
use pumpkin_core::math::vector2::Vector2;

use crate::{
    biome::Biome,
    block::block_state::BlockState,
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
    fn generate_block(&self, at: BlockCoordinates, chunk_height: i16, _: Biome) -> BlockState {
        let begin_stone_height = chunk_height - 5;
        let begin_dirt_height = chunk_height - 1;

        let y = *at.y;
        if y == -64 {
            pumpkin_macros::block!("minecraft:bedrock")
        } else if y >= -63 && y <= begin_stone_height {
            pumpkin_macros::block!("minecraft:stone")
        } else if y >= begin_stone_height && y < begin_dirt_height {
            pumpkin_macros::block!("minecraft:dirt")
        } else if y == chunk_height - 1 {
            pumpkin_macros::block!("minecraft:grass_block")
        } else {
            BlockState::AIR
        }
    }
}
