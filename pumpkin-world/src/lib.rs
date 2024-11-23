use block::BlockState;
use world_gen::{
    chunk_noise::{ChunkNoiseGenerator, LAVA_BLOCK, WATER_BLOCK},
    generation_shapes::GenerationShape,
    noise::{config::NoiseConfig, router::OVERWORLD_NOISE_ROUTER},
    proto_chunk::StandardChunkFluidLevelSampler,
    sampler::{FluidLevel, FluidLevelSampler},
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

// TODO: is there a way to do in-file benches?
pub fn bench_create_chunk_noise_overworld() {
    let config = NoiseConfig::new(0, &OVERWORLD_NOISE_ROUTER);
    let generation_shape = GenerationShape::SURFACE;
    let sampler = FluidLevelSampler::Chunk(StandardChunkFluidLevelSampler {
        bottom_fluid: FluidLevel::new(-54, *LAVA_BLOCK),
        top_fluid: FluidLevel::new(62, *WATER_BLOCK),
    });

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
    let config = NoiseConfig::new(0, &OVERWORLD_NOISE_ROUTER);
    let fluid_sampler = FluidLevelSampler::Chunk(StandardChunkFluidLevelSampler {
        bottom_fluid: FluidLevel::new(-54, *LAVA_BLOCK),
        top_fluid: FluidLevel::new(62, *WATER_BLOCK),
    });
    let generation_shape = GenerationShape::SURFACE;

    let mut sampler = ChunkNoiseGenerator::new(
        16 / generation_shape.horizontal_cell_block_count(),
        0,
        0,
        generation_shape,
        &config,
        fluid_sampler,
        true,
        true,
    );

    let horizontal_cell_block_count = sampler.horizontal_cell_block_count();
    let vertical_cell_block_count = sampler.vertical_cell_block_count();

    let horizonal_cells = 16 / horizontal_cell_block_count;

    let min_y = sampler.min_y();
    let minimum_cell_y = min_y / vertical_cell_block_count as i8;
    let cell_height = sampler.height() / vertical_cell_block_count as u16;

    sampler.sample_start_density();
    for cell_x in 0..horizonal_cells {
        sampler.sample_end_density(cell_x);

        for cell_z in 0..horizonal_cells {
            for cell_y in (0..cell_height).rev() {
                sampler.on_sampled_cell_corners(cell_y, cell_z);
                for local_y in (0..vertical_cell_block_count).rev() {
                    let block_y = (minimum_cell_y as i32 + cell_y as i32)
                        * vertical_cell_block_count as i32
                        + local_y as i32;
                    let delta_y = local_y as f64 / vertical_cell_block_count as f64;
                    sampler.interpolate_y(block_y, delta_y);

                    for local_x in 0..horizontal_cell_block_count {
                        let block_x =
                            cell_x as i32 * horizontal_cell_block_count as i32 + local_x as i32;
                        let delta_x = local_x as f64 / horizontal_cell_block_count as f64;
                        sampler.interpolate_x(block_x, delta_x);

                        for local_z in 0..horizontal_cell_block_count {
                            let block_z =
                                cell_z as i32 * horizontal_cell_block_count as i32 + local_z as i32;
                            let delta_z = local_z as f64 / horizontal_cell_block_count as f64;
                            sampler.interpolate_z(block_z, delta_z);

                            // TODO: Change default block
                            let _block_state = sampler
                                .sample_block_state()
                                .unwrap_or(BlockState::new("minecraft:stone").unwrap());
                            //log::debug!("Sampled block state in {:?}", inst.elapsed());

                            //println!("Putting {:?}: {:?}", local_pos, block_state);
                            //self.block_map.insert(local_pos, block_state);
                        }
                    }
                }
            }
        }

        sampler.swap_buffers();
    }

    sampler.stop_interpolation();
}
