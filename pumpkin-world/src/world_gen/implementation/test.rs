use std::{
    num::Wrapping,
    ops::{AddAssign, SubAssign},
};

use dashmap::{DashMap, Entry};
use num_traits::Zero;
use pumpkin_core::math::{vector2::Vector2, vector3::Vector3};

use crate::{
    biome::Biome,
    block::block_state::BlockState,
    chunk::{ChunkBlocks, ChunkData},
    coordinates::{
        ChunkRelativeBlockCoordinates, ChunkRelativeXZBlockCoordinates, XZBlockCoordinates,
    },
    world_gen::{
        generator::{BiomeGenerator, GeneratorInit, TerrainGenerator},
        proto_chunk::ProtoChunk,
        Seed, WorldGenerator,
    },
    WORLD_LOWEST_Y, WORLD_MAX_Y,
};

pub struct TestGenerator<B: BiomeGenerator, T: TerrainGenerator> {
    biome_generator: B,
    terrain_generator: T,
}

impl<B: BiomeGenerator + GeneratorInit, T: TerrainGenerator + GeneratorInit> GeneratorInit
    for TestGenerator<B, T>
{
    fn new(seed: Seed) -> Self {
        Self {
            biome_generator: B::new(seed),
            terrain_generator: T::new(seed),
        }
    }
}

impl<B: BiomeGenerator, T: TerrainGenerator> WorldGenerator for TestGenerator<B, T> {
    fn generate_chunk(&self, at: Vector2<i32>) -> ChunkData {
        let mut blocks = ChunkBlocks::default();
        self.terrain_generator.prepare_chunk(&at);

        for x in 0..16u8 {
            for z in 0..16u8 {
                let biome = self.biome_generator.generate_biome(
                    ChunkRelativeXZBlockCoordinates {
                        x: x.into(),
                        z: z.into(),
                    }
                    .with_chunk_coordinates(at),
                );

                // TODO: This can be chunk specific
                for y in (WORLD_LOWEST_Y..WORLD_MAX_Y).rev() {
                    let coordinates = ChunkRelativeBlockCoordinates {
                        x: x.into(),
                        y: y.into(),
                        z: z.into(),
                    };

                    let block = self.terrain_generator.generate_block(
                        &at,
                        Vector3::new(x.into(), y.into(), z.into()),
                        biome,
                    );

                    //println!("{:?}: {:?}", coordinates, block);
                    blocks.set_block(coordinates, block.state_id);
                }
            }
        }

        self.terrain_generator.clean_chunk(&at);
        ChunkData {
            blocks,
            position: at,
        }
    }
}

pub(crate) struct TestBiomeGenerator {}

impl GeneratorInit for TestBiomeGenerator {
    fn new(_: Seed) -> Self {
        Self {}
    }
}

impl BiomeGenerator for TestBiomeGenerator {
    // TODO make generic over Biome and allow changing the Biome in the config.
    fn generate_biome(&self, _: XZBlockCoordinates) -> Biome {
        Biome::Plains
    }
}

pub(crate) struct TestTerrainGenerator {
    chunks: DashMap<Vector2<i32>, (ProtoChunk, Wrapping<u8>)>,
    seed: Seed,
}

impl GeneratorInit for TestTerrainGenerator {
    fn new(seed: Seed) -> Self {
        Self {
            chunks: DashMap::new(),
            seed,
        }
    }
}

impl TerrainGenerator for TestTerrainGenerator {
    fn prepare_chunk(&self, at: &Vector2<i32>) {
        let entry = self.chunks.entry(*at);
        match entry {
            Entry::Vacant(entry) => {
                let mut proto_chunk = ProtoChunk::new(*at, self.seed.0);
                //let inst = std::time::Instant::now();
                //println!("Populating chunk: {:?}", at);
                proto_chunk.populate_noise();
                //println!("Done populating chunk: {:?} ({:?})", at, inst.elapsed());
                entry.insert((proto_chunk, Wrapping(1)));
            }
            Entry::Occupied(mut entry) => {
                let (_, count) = entry.get_mut();
                count.add_assign(1);
            }
        }
    }

    fn clean_chunk(&self, at: &Vector2<i32>) {
        let entry = self.chunks.entry(*at);
        if let Entry::Occupied(mut entry) = entry {
            let (_, count) = entry.get_mut();
            count.sub_assign(1);
            if count.is_zero() {
                entry.remove();
            }
        }
    }

    // TODO allow specifying which blocks should be at which height in the config.
    fn generate_block(
        &self,
        chunk_pos: &Vector2<i32>,
        local_pos: Vector3<i32>,
        _: Biome,
    ) -> BlockState {
        if let Some(entry) = self.chunks.get(chunk_pos) {
            entry.0.get_block_state(&local_pos)
        } else {
            panic!("Chunk needs to exist")
        }
    }
}
