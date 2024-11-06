use noise::{NoiseFn, Perlin};
use pumpkin_core::math::vector2::Vector2;

use crate::{
    chunk::{ChunkBlocks, ChunkData, ChunkHeightmaps},
    coordinates::{ChunkRelativeBlockCoordinates, ChunkRelativeXZBlockCoordinates},
    WORLD_LOWEST_Y,
};

use super::{
    generator::{BiomeGenerator, GeneratorInit, PerlinTerrainGenerator, WorldGenerator},
    Seed,
};

pub struct GenericGenerator<B: BiomeGenerator, T: PerlinTerrainGenerator> {
    biome_generator: B,
    terrain_generator: T,
    // TODO: May make this optional?. But would be pain to use in most biomes then. Maybe make a new trait like
    // PerlinTerrainGenerator
    perlin: Perlin,
}

impl<B: BiomeGenerator + GeneratorInit, T: PerlinTerrainGenerator + GeneratorInit> GeneratorInit
    for GenericGenerator<B, T>
{
    fn new(seed: Seed) -> Self {
        Self {
            biome_generator: B::new(seed),
            terrain_generator: T::new(seed),
            perlin: Perlin::new(seed.0 as u32),
        }
    }
}

impl<B: BiomeGenerator, T: PerlinTerrainGenerator> WorldGenerator for GenericGenerator<B, T> {
    fn generate_chunk(&self, at: Vector2<i32>) -> ChunkData {
        let mut blocks = ChunkBlocks::default();
        self.terrain_generator.prepare_chunk(&at, &self.perlin);
        let noise_value = self.perlin.get([at.x as f64 / 16.0, at.z as f64 / 16.0]);

        let base_height = 64.0;
        let chunk_height =
            noise_value.mul_add(self.terrain_generator.height_variation(), base_height) as i16;

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
                for y in (WORLD_LOWEST_Y..chunk_height).rev() {
                    let coordinates = ChunkRelativeBlockCoordinates {
                        x: x.into(),
                        y: y.into(),
                        z: z.into(),
                    };

                    //coordinates,
                    self.terrain_generator.generate_block(
                        coordinates,
                        coordinates.with_chunk_coordinates(at),
                        &mut blocks,
                        chunk_height,
                        biome,
                    );
                }
            }
        }

        ChunkData {
            blocks,
            position: at,
            heightmap: ChunkHeightmaps::default()
        }
    }
}

// TODO: implement static terrain generator
/*
fn generate_chunk(&mut self, at: Vector2<i32>) -> ChunkData {
    let mut blocks = ChunkBlocks::default();
    self.terrain_generator.prepare_chunk(&at, &self.perlin);
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
*/
