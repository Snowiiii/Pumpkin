use noise::Perlin;
use pumpkin_core::math::vector2::Vector2;
use pumpkin_macros::block_state_id;
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
            blocks.set_block(coordinates, block_state_id!("minecraft:bedrock"));
        } else if y >= -63 && y <= begin_stone_height {
            blocks.set_block(coordinates, block_state_id!("minecraft:stone"));
        } else if y >= begin_stone_height && y < begin_dirt_height {
            blocks.set_block(coordinates, block_state_id!("minecraft:dirt"));
        } else if y == chunk_height - 2 {
            blocks.set_block(coordinates, block_state_id!("minecraft:grass_block"));
        } else if y == chunk_height - 1 {
            // TODO: generate flowers and grass
            let grass: u8 = rand::thread_rng().gen_range(0..7);
            if grass == 3 {
                let flower: u8 = rand::thread_rng().gen_range(0..20);
                if flower == 6 {
                    match rand::thread_rng().gen_range(0..4) {
                        0 => {
                            blocks.set_block(coordinates, block_state_id!("minecraft:dandelion"));
                        }
                        1 => {
                            blocks.set_block(coordinates, block_state_id!("minecraft:oxeye_daisy"));
                        }
                        2 => {
                            blocks.set_block(coordinates, block_state_id!("minecraft:cornflower"));
                        }
                        3 => {
                            blocks.set_block(coordinates, block_state_id!("minecraft:poppy"));
                        }
                        _ => {
                            blocks.set_block(coordinates, block_state_id!("minecraft:azure_bluet"));
                        }
                    }
                } else {
                    // TODO: Tall grass, Tall grass data called `half`, There is `upper` and `lower`
                    blocks.set_block(coordinates, block_state_id!("minecraft:short_grass"));
                }
            }
        }
        //  BlockState::AIR
    }
}
