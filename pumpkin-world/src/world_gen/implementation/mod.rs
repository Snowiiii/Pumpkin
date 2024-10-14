use super::{
    blender::Blender,
    chunk::{Chunk, GenerationShapeConfig},
    noise::config::NoiseConfig,
};

mod aquifer;
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
}
