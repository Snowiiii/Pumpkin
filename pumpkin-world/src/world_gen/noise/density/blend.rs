use std::sync::Arc;

use crate::world_gen::blender::BlenderImpl;

use super::{
    Applier, ApplierImpl, DensityFunction, DensityFunctionImpl, NoisePos, NoisePosImpl, Visitor,
    VisitorImpl,
};

#[derive(Clone)]
pub struct BlendOffsetFunction {}

impl<'a> DensityFunctionImpl<'a> for BlendOffsetFunction {
    fn sample(&self, _pos: &NoisePos) -> f64 {
        0f64
    }

    fn fill(&self, densities: &[f64], _applier: &Applier) -> Vec<f64> {
        densities.iter().map(|_| 0f64).collect()
    }

    fn min(&self) -> f64 {
        0f64
    }

    fn max(&self) -> f64 {
        0f64
    }

    fn apply(&'a self, visitor: &'a Visitor) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::BlendOffset(self.clone())))
    }
}

#[derive(Clone)]
pub struct BlendAlphaFunction {}

impl<'a> DensityFunctionImpl<'a> for BlendAlphaFunction {
    fn sample(&self, _pos: &NoisePos) -> f64 {
        1f64
    }

    fn fill(&self, densities: &[f64], _applier: &Applier) -> Vec<f64> {
        densities.iter().map(|_| 1f64).collect()
    }

    fn max(&self) -> f64 {
        1f64
    }

    fn min(&self) -> f64 {
        1f64
    }

    fn apply(&'a self, visitor: &'a Visitor) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::BlendAlpha(self.clone())))
    }
}

#[derive(Clone)]
pub struct BlendDensityFunction<'a> {
    function: Arc<DensityFunction<'a>>,
}

impl<'a> BlendDensityFunction<'a> {
    pub fn new(density: Arc<DensityFunction<'a>>) -> Self {
        Self { function: density }
    }
}

impl<'a> BlendDensityFunction<'a> {
    fn apply_density(&self, pos: &NoisePos, density: f64) -> f64 {
        pos.get_blender().apply_blend_density(pos, density)
    }
}

impl<'a> DensityFunctionImpl<'a> for BlendDensityFunction<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        self.apply_density(pos, self.function.sample(pos))
    }

    fn fill(&self, densities: &[f64], applier: &Applier) -> Vec<f64> {
        let densities = self.function.fill(densities, applier);
        densities
            .iter()
            .enumerate()
            .map(|(i, x)| self.apply_density(&applier.at(i as i32), *x))
            .collect()
    }

    fn apply(&'a self, visitor: &'a Visitor) -> Arc<DensityFunction<'a>> {
        let new_function = BlendDensityFunction {
            function: self.function.apply(visitor),
        };
        visitor.apply(Arc::new(DensityFunction::BlendDensity(new_function)))
    }

    fn min(&self) -> f64 {
        f64::NEG_INFINITY
    }

    fn max(&self) -> f64 {
        f64::INFINITY
    }
}
