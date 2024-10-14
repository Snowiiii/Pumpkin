use std::sync::Arc;

use super::{
    noise::InternalNoise, Applier, ApplierImpl, DensityFunction, DensityFunctionImpl, NoisePos,
    NoisePosImpl, OffsetDensityFunction, Visitor, VisitorImpl,
};

#[derive(Clone)]
pub struct ShiftAFunction<'a> {
    offset: Arc<InternalNoise<'a>>,
}

impl<'a> ShiftAFunction<'a> {
    pub fn new(offset: Arc<InternalNoise<'a>>) -> Self {
        Self { offset }
    }
}

impl<'a> OffsetDensityFunction<'a> for ShiftAFunction<'a> {
    fn offset_noise(&self) -> &InternalNoise<'a> {
        &self.offset
    }
}

impl<'a> DensityFunctionImpl<'a> for ShiftAFunction<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        self.sample_3d(pos.x() as f64, 0f64, pos.z() as f64)
    }

    fn apply(&'a self, visitor: &'a Visitor) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::ShiftA(ShiftAFunction {
            offset: visitor.apply_internal_noise(self.offset.clone()),
        })))
    }

    fn fill(&self, densities: &[f64], applier: &Applier) -> Vec<f64> {
        applier.fill(densities, &DensityFunction::ShiftA(self.clone()))
    }

    fn max(&self) -> f64 {
        self.offset_noise().max_value() * 4f64
    }

    fn min(&self) -> f64 {
        -self.max()
    }
}

#[derive(Clone)]
pub struct ShiftBFunction<'a> {
    offset: Arc<InternalNoise<'a>>,
}

impl<'a> ShiftBFunction<'a> {
    pub fn new(offset: Arc<InternalNoise<'a>>) -> Self {
        Self { offset }
    }
}

impl<'a> OffsetDensityFunction<'a> for ShiftBFunction<'a> {
    fn offset_noise(&self) -> &InternalNoise<'a> {
        &self.offset
    }
}

impl<'a> DensityFunctionImpl<'a> for ShiftBFunction<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        self.sample_3d(pos.z() as f64, pos.x() as f64, 0f64)
    }

    fn apply(&'a self, visitor: &'a Visitor) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::ShiftB(ShiftBFunction {
            offset: visitor.apply_internal_noise(self.offset.clone()),
        })))
    }

    fn fill(&self, densities: &[f64], applier: &Applier) -> Vec<f64> {
        applier.fill(densities, &DensityFunction::ShiftB(self.clone()))
    }

    fn max(&self) -> f64 {
        self.offset_noise().max_value() * 4f64
    }

    fn min(&self) -> f64 {
        -self.max()
    }
}
