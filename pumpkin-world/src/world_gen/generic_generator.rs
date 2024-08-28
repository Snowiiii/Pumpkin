use crate::{
    chunk::{ChunkBlocks, ChunkData},
    coordinates::{
        ChunkCoordinates, ChunkRelativeBlockCoordinates, ChunkRelativeXZBlockCoordinates,
    },
    WORLD_LOWEST_Y, WORLD_MAX_Y,
};

use super::{
    generator::{BiomeGenerator, GeneratorInit, TerrainGenerator, WorldGenerator},
    Seed,
};

pub struct GenericGenerator<B: BiomeGenerator, T: TerrainGenerator> {
    biome_generator: B,
    terrain_generator: T,
}

impl<B: BiomeGenerator + GeneratorInit, T: TerrainGenerator + GeneratorInit> GeneratorInit
    for GenericGenerator<B, T>
{
    fn new(seed: Seed) -> Self {
        Self {
            biome_generator: B::new(seed),
            terrain_generator: T::new(seed),
        }
    }
}

impl<B: BiomeGenerator, T: TerrainGenerator> WorldGenerator for GenericGenerator<B, T> {
    fn generate_chunk(&self, at: ChunkCoordinates) -> ChunkData {
        let mut blocks = ChunkBlocks::default();

        for x in 0..16u8 {
            for z in 0..16u8 {
                let biome = self.biome_generator.generate_biome(
                    ChunkRelativeXZBlockCoordinates {
                        x: x.into(),
                        z: z.into(),
                    }
                    .with_chunk_coordinates(at),
                );

                // Iterate from the highest block to the lowest, in order to minimize the heightmap updates
                for y in (WORLD_LOWEST_Y..WORLD_MAX_Y).rev() {
                    let coordinates = ChunkRelativeBlockCoordinates {
                        x: x.into(),
                        y: y.into(),
                        z: z.into(),
                    };

                    blocks.set_block(
                        coordinates,
                        self.terrain_generator
                            .generate_block(coordinates.with_chunk_coordinates(at), biome),
                    );
                }
            }
        }

        ChunkData {
            blocks,
            position: at,
        }
    }
}
