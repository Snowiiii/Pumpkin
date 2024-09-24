use crate::height::HeightLimitViewImpl;

use super::{
    blender::Blender,
    chunk::{Chunk, GenerationShapeConfig, HeightMapType},
    noise::config::NoiseConfig,
};

pub mod aquifer;
pub mod overworld;
pub mod superflat;
mod surface_builder;

pub fn populate_noise(
    chunk: &mut Chunk,
    blender: &Blender,
    config: &NoiseConfig,
    shape: &GenerationShapeConfig,
) {
    let shape = shape.trim_height(chunk);
    let i = shape.min_y();
    let j = i / shape.vertical_cell_block_count();
    let k = shape.height() / shape.vertical_cell_block_count();

    internal_populate_noise(chunk, blender, config, j, k);
}

fn internal_populate_noise(
    chunk: &mut Chunk,
    blender: &Blender,
    config: &NoiseConfig,
    min_cell_y: i32,
    cell_height: i32,
) {
    let sampler = chunk.get_or_create_noise_sampler();
    let height_map = chunk.get_height_map(HeightMapType::WorldGenOceanFloor);
    let height_map2 = chunk.get_height_map(HeightMapType::WorldGenSurface);
    let pos = chunk.pos();

    let i = pos.get_start_x();
    let j = pos.get_start_z();

    let mut sampler = sampler.lock();
    sampler.sample_start_density();

    let k = sampler.horizontal_cell_block_count();
    let l = sampler.vertical_cell_block_count();
    let m = 16 / k;
    let n = 16 / k;

    for o in 0..m {
        sampler.sample_end_density(o);

        for p in 0..n {
            let q = chunk.vertical_section_count() - 1;
        }
    }
}
