use pumpkin_core::math::vector2::Vector2;
use world_gen::{
    aquifer_sampler::{FluidLevel, FluidLevelSampler},
    chunk_noise::{ChunkNoiseGenerator, LAVA_BLOCK, WATER_BLOCK},
    generation_shapes::GenerationShape,
    noise::{config::NoiseConfig, router::OVERWORLD_NOISE_ROUTER},
    proto_chunk::{ProtoChunk, StandardChunkFluidLevelSampler},
};

pub mod biome;
pub mod block;
pub mod chunk;
pub mod coordinates;
pub mod cylindrical_chunk_iterator;
pub mod dimension;
pub mod item;
pub mod level;
mod world_gen;

pub const WORLD_HEIGHT: usize = 384;
pub const WORLD_LOWEST_Y: i16 = -64;
pub const WORLD_MAX_Y: i16 = WORLD_HEIGHT as i16 - WORLD_LOWEST_Y.abs();
pub const DIRECT_PALETTE_BITS: u32 = 15;

#[macro_export]
macro_rules! read_data_from_file {
    ($path:expr) => {
        serde_json::from_str(
            &fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .join(file!())
                    .parent()
                    .unwrap()
                    .join($path),
            )
            .expect("no data file"),
        )
        .expect("failed to decode array")
    };
}

// TODO: is there a way to do in-file benches?
pub fn bench_create_chunk_noise_overworld() {
    let config = NoiseConfig::new(0, &OVERWORLD_NOISE_ROUTER);
    let generation_shape = GenerationShape::SURFACE;
    let sampler = FluidLevelSampler::Chunk(StandardChunkFluidLevelSampler::new(
        FluidLevel::new(63, *WATER_BLOCK),
        FluidLevel::new(-54, *LAVA_BLOCK),
    ));

    ChunkNoiseGenerator::new(
        16 / generation_shape.horizontal_cell_block_count(),
        0,
        0,
        generation_shape,
        &config,
        sampler,
        true,
        true,
    );
}

pub fn bench_create_and_populate_noise() {
    let mut chunk = ProtoChunk::new(Vector2::new(0, 0), 0);
    chunk.populate_noise();
}
