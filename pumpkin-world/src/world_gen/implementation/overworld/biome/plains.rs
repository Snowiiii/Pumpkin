use noise::Perlin;
use pumpkin_core::math::vector2::Vector2;
use rand::Rng;

use crate::{
    biome::Biome,
    chunk::ChunkBlocks,
    coordinates::{BlockCoordinates, ChunkRelativeBlockCoordinates, XZBlockCoordinates},
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
    fn generate_block(
        &self,
        coordinates: ChunkRelativeBlockCoordinates,
        at: BlockCoordinates,
        blocks: &mut ChunkBlocks,
        chunk_height: i16,
        _: Biome,
    ) {
        let begin_stone_height = chunk_height - 5;
        let begin_dirt_height = chunk_height - 2;

        let y = *at.y;
        if y == -64 {
            blocks.set_block(
                coordinates,
                pumpkin_macros::block!("minecraft:bedrock").into(),
            );
        } else if y >= -63 && y <= begin_stone_height {
            blocks.set_block(
                coordinates,
                pumpkin_macros::block!("minecraft:stone").into(),
            );
        } else if y >= begin_stone_height && y < begin_dirt_height {
            blocks.set_block(coordinates, pumpkin_macros::block!("minecraft:dirt").into());
        } else if y == chunk_height - 2 {
            blocks.set_block(
                coordinates,
                pumpkin_macros::block!("minecraft:grass_block").into(),
            );
        } else if y == chunk_height - 1 {
            // TODO: generate flowers and grass
            let grass: u8 = rand::thread_rng().gen_range(0..7);
            if grass == 3 {
                let flower: u8 = rand::thread_rng().gen_range(0..20);
                if flower == 6 {
                    match rand::thread_rng().gen_range(0..4) {
                        0 => {
                            blocks.set_block(
                                coordinates,
                                pumpkin_macros::block!("minecraft:dandelion").into(),
                            );
                        }
                        1 => {
                            blocks.set_block(
                                coordinates,
                                pumpkin_macros::block!("minecraft:oxeye_daisy").into(),
                            );
                        }
                        2 => {
                            blocks.set_block(
                                coordinates,
                                pumpkin_macros::block!("minecraft:cornflower").into(),
                            );
                        }
                        3 => {
                            blocks.set_block(
                                coordinates,
                                pumpkin_macros::block!("minecraft:poppy").into(),
                            );
                        }
                        _ => {
                            blocks.set_block(
                                coordinates,
                                pumpkin_macros::block!("minecraft:azure_bluet").into(),
                            );
                        }
                    }
                } else {
                    // TODO: Tall grass, Tall grass data called `half`, There is `upper` and `lower`
                    blocks.set_block(
                        coordinates,
                        pumpkin_macros::block!("minecraft:short_grass").into(),
                    );
                }
            }
        }
        //  BlockState::AIR
    }
}
