use std::rc::Rc;

use super::{noise::InternalNoise, DensityFunction, DensityFunctionImpl, OffsetDensityFunction};

#[derive(Clone)]
pub struct ShiftAFunction<'a> {
    offset: Rc<InternalNoise<'a>>,
}

impl<'a> OffsetDensityFunction<'a> for ShiftAFunction<'a> {
    fn offset_noise(&self) -> &InternalNoise<'a> {
        &self.offset
    }
}

impl<'a> DensityFunctionImpl<'a> for ShiftAFunction<'a> {
    fn sample(&self, pos: &impl super::NoisePos) -> f64 {
        self.sample_3d(pos.x() as f64, 0f64, pos.z() as f64)
    }

    fn apply(&'a self, visitor: &'a impl super::Visitor) -> super::DensityFunction<'a> {
        visitor.apply(&DensityFunction::ShiftA(ShiftAFunction {
            offset: visitor.apply_internal_noise(self.offset.clone()),
        }))
    }

    fn fill(&self, densities: &[f64], applier: &impl super::Applier) -> Vec<f64> {
        applier.fill(densities, self)
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
    offset: Rc<InternalNoise<'a>>,
}

impl<'a> OffsetDensityFunction<'a> for ShiftBFunction<'a> {
    fn offset_noise(&self) -> &InternalNoise<'a> {
        &self.offset
    }
}

impl<'a> DensityFunctionImpl<'a> for ShiftBFunction<'a> {
    fn sample(&self, pos: &impl super::NoisePos) -> f64 {
        self.sample_3d(pos.z() as f64, pos.x() as f64, 0f64)
    }

    fn apply(&'a self, visitor: &'a impl super::Visitor) -> super::DensityFunction<'a> {
        visitor.apply(&DensityFunction::ShiftB(ShiftBFunction {
            offset: visitor.apply_internal_noise(self.offset.clone()),
        }))
    }

    fn fill(&self, densities: &[f64], applier: &impl super::Applier) -> Vec<f64> {
        applier.fill(densities, self)
    }

    fn max(&self) -> f64 {
        self.offset_noise().max_value() * 4f64
    }

    fn min(&self) -> f64 {
        -self.max()
    }
}
