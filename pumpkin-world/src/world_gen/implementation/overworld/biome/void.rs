use crate::{biome::Biome, chunk::ChunkData, coordinates::XZBlockCoordinates, world_gen::{generator::{BiomeGenerator, GeneratorInit}, generic_generator::GenericGenerator, Seed, WorldGenerator}};


// TODO
// Fix generation, because void generator dont needs perlin generator
// only world generator
pub type VoidGenerator = GenericGenerator<VoidBiomeGenerator, VoidTerrainGenerator>;

pub(crate) struct VoidBiomeGenerator {}

impl GeneratorInit for VoidBiomeGenerator {
    fn new(_: Seed) -> Self {
        Self {}
    }
}

impl BiomeGenerator for VoidBiomeGenerator {
    fn generate_biome(&self, _: XZBlockCoordinates) -> Biome {
        Biome::Void
    }
}

pub(crate) struct VoidTerrainGenerator {}

impl GeneratorInit for VoidTerrainGenerator {
    fn new(_: Seed) -> Self {
        Self {}
    }
}

impl WorldGenerator for VoidTerrainGenerator {
    fn generate_chunk(&self, position: pumpkin_core::math::vector2::Vector2<i32>) -> ChunkData {
        ChunkData { position, ..Default::default() }
    }
}