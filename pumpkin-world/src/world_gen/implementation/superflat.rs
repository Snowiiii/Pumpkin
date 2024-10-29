use pumpkin_core::math::vector2::Vector2;

use crate::{
    biome::Biome,
    block::block_state::BlockState,
    coordinates::{BlockCoordinates, XZBlockCoordinates},
    world_gen::{
        generator::{BiomeGenerator, GeneratorInit, TerrainGenerator},
        generic_generator::GenericGenerator,
        Seed,
    },
};

#[expect(dead_code)]
pub type SuperflatGenerator = GenericGenerator<SuperflatBiomeGenerator, SuperflatTerrainGenerator>;

pub(crate) struct SuperflatBiomeGenerator {}

impl GeneratorInit for SuperflatBiomeGenerator {
    fn new(_: Seed) -> Self {
        Self {}
    }
}

impl BiomeGenerator for SuperflatBiomeGenerator {
    // TODO make generic over Biome and allow changing the Biome in the config.
    fn generate_biome(&self, _: XZBlockCoordinates) -> Biome {
        Biome::Plains
    }
}

pub(crate) struct SuperflatTerrainGenerator {}

impl GeneratorInit for SuperflatTerrainGenerator {
    fn new(_: Seed) -> Self {
        Self {}
    }
}

impl TerrainGenerator for SuperflatTerrainGenerator {
    fn prepare_chunk(&self, _at: &Vector2<i32>) {}
    // TODO allow specifying which blocks should be at which height in the config.
    fn generate_block(&self, at: BlockCoordinates, _: Biome) -> BlockState {
        match *at.y {
            -64 => BlockState::new("minecraft:bedrock").unwrap(),
            -63..=-62 => BlockState::new("minecraft:dirt").unwrap(),
            -61 => BlockState::new("minecraft:grass_block").unwrap(),
            _ => BlockState::AIR,
        }
    }
}
