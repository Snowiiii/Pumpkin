use std::rc::Rc;

use crate::world_gen::noise::perlin::{DoublePerlinNoiseParameters, DoublePerlinNoiseSampler};

use super::{DensityFunction, DensityFunctionImpl};

pub(crate) struct InternalNoise<'a> {
    data: DoublePerlinNoiseParameters<'a>,
    sampler: Option<DoublePerlinNoiseSampler>,
}

impl<'a> InternalNoise<'a> {
    pub(crate) fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        match &self.sampler {
            Some(sampler) => sampler.sample(x, y, z),
            None => 0f64,
        }
    }

    pub(crate) fn max_value(&self) -> f64 {
        match &self.sampler {
            Some(sampler) => sampler.max_value(),
            None => 2f64,
        }
    }
}

#[derive(Clone)]
pub struct NoiseFunction<'a> {
    noise: Rc<InternalNoise<'a>>,
    xz_scale: f64,
    y_scale: f64,
}

impl<'a> DensityFunctionImpl<'a> for NoiseFunction<'a> {
    fn sample(&self, pos: &impl super::NoisePos) -> f64 {
        self.noise.sample(
            pos.x() as f64 * self.xz_scale,
            pos.y() as f64 * self.y_scale,
            pos.z() as f64 * self.xz_scale,
        )
    }

    fn fill(&self, densities: &[f64], applier: &impl super::Applier) -> Vec<f64> {
        applier.fill(densities, self)
    }

    fn apply(&self, visitor: &'a impl super::Visitor) -> super::DensityFunction<'a> {
        visitor.apply(&super::DensityFunction::Noise(self.clone()))
    }

    fn max(&self) -> f64 {
        self.noise.max_value()
    }

    fn min(&self) -> f64 {
        -self.max()
    }
}

#[derive(Clone)]
pub struct ShiftedNoiseFunction<'a> {
    shift_x: Rc<DensityFunction<'a>>,
    shift_y: Rc<DensityFunction<'a>>,
    shift_z: Rc<DensityFunction<'a>>,
    noise: Rc<InternalNoise<'a>>,
    xz_scale: f64,
    y_scale: f64,
}

impl<'a> DensityFunctionImpl<'a> for ShiftedNoiseFunction<'a> {
    fn sample(&self, pos: &impl super::NoisePos) -> f64 {
        let d = pos.x() as f64 * self.xz_scale + self.shift_x.sample(pos);
        let e = pos.y() as f64 * self.y_scale + self.shift_y.sample(pos);
        let f = pos.z() as f64 * self.xz_scale + self.shift_z.sample(pos);

        self.noise.sample(d, e, f)
    }

    fn fill(&self, densities: &[f64], applier: &impl super::Applier) -> Vec<f64> {
        applier.fill(densities, self)
    }

    fn apply(&'a self, visitor: &'a impl super::Visitor) -> DensityFunction<'a> {
        let new_x = self.shift_x.apply(visitor);
        let new_y = self.shift_y.apply(visitor);
        let new_z = self.shift_z.apply(visitor);
        let new_noise = visitor.apply_internal_noise(self.noise.clone());

        DensityFunction::ShiftedNoise(ShiftedNoiseFunction {
            shift_x: Rc::new(new_x),
            shift_y: Rc::new(new_y),
            shift_z: Rc::new(new_z),
            xz_scale: self.xz_scale,
            y_scale: self.y_scale,
            noise: new_noise,
        })
    }

    fn max(&self) -> f64 {
        self.noise.max_value()
    }

    fn min(&self) -> f64 {
        -self.max()
    }
}
