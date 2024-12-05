use std::sync::Arc;

use super::{
    component_functions::{
        ApplierImpl, ComponentFunctionImpl, ImmutableComponentFunctionImpl,
        SharedConverterEnvironment,
    },
    noise::InternalNoise,
    NoisePos, NoisePosImpl,
};

#[inline]
fn sample_3d(noise: &InternalNoise, x: f64, y: f64, z: f64) -> f64 {
    noise.sample(x * 0.25f64, y * 0.25f64, z * 0.25f64) * 4f64
}

pub struct ShiftAFunction {
    pub(crate) offset: Arc<InternalNoise>,
}

impl ShiftAFunction {
    pub fn new(offset: Arc<InternalNoise>) -> Self {
        Self { offset }
    }
}

impl ComponentFunctionImpl for ShiftAFunction {}

impl ImmutableComponentFunctionImpl for ShiftAFunction {
    #[inline]
    fn sample(&self, pos: &NoisePos) -> f64 {
        sample_3d(&self.offset, pos.x() as f64, 0f64, pos.z() as f64)
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        applier.fill(arr, self);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::ShiftA(&self.offset)
    }
}

pub struct ShiftBFunction {
    pub(crate) offset: Arc<InternalNoise>,
}

impl ShiftBFunction {
    pub fn new(offset: Arc<InternalNoise>) -> Self {
        Self { offset }
    }
}

impl ComponentFunctionImpl for ShiftBFunction {}

impl ImmutableComponentFunctionImpl for ShiftBFunction {
    #[inline]
    fn sample(&self, pos: &NoisePos) -> f64 {
        sample_3d(&self.offset, pos.z() as f64, pos.x() as f64, 0f64)
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        applier.fill(arr, self);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::ShiftB(&self.offset)
    }
}
