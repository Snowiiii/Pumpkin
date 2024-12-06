use enum_dispatch::enum_dispatch;

use super::noise::density::NoisePos;

pub struct BlendResult {
    alpha: f64,
    offset: f64,
}

impl BlendResult {
    pub fn new(alpha: f64, offset: f64) -> Self {
        Self { alpha, offset }
    }
}

#[enum_dispatch(BlenderImpl)]
pub enum Blender {
    NoBlend(NoBlendBlender),
}

impl Blender {
    pub const NO_BLEND: Self = Self::NoBlend(NoBlendBlender {});
}

#[enum_dispatch]
pub trait BlenderImpl {
    fn calculate(&self, block_x: i32, block_z: i32) -> BlendResult;

    fn apply_blend_density(&self, pos: &NoisePos, density: f64) -> f64;

    fn get_biome_supplier(&self) {
        todo!()
    }
}

pub struct NoBlendBlender {}

impl BlenderImpl for NoBlendBlender {
    fn calculate(&self, _block_x: i32, _block_z: i32) -> BlendResult {
        BlendResult::new(1f64, 1f64)
    }

    fn apply_blend_density(&self, _pos: &NoisePos, density: f64) -> f64 {
        density
    }
}
