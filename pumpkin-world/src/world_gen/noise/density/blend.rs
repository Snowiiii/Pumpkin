use std::sync::Arc;

use super::{DensityFunction, DensityFunctionImpl, NoisePos};

#[derive(Clone)]
pub struct BlendOffsetFunction {}

impl<'a> DensityFunctionImpl<'a> for BlendOffsetFunction {
    fn sample(&self, _pos: &impl super::NoisePos) -> f64 {
        0f64
    }

    fn fill(&self, densities: &[f64], _applier: &impl super::Applier) -> Vec<f64> {
        densities.iter().map(|_| 0f64).collect()
    }

    fn min(&self) -> f64 {
        0f64
    }

    fn max(&self) -> f64 {
        0f64
    }

    fn apply(&'a self, visitor: &'a impl super::Visitor) -> super::DensityFunction<'a> {
        visitor.apply(&DensityFunction::BlendOffset(self.clone()))
    }
}

#[derive(Clone)]
pub struct BlendAlphaFunction {}

impl<'a> DensityFunctionImpl<'a> for BlendAlphaFunction {
    fn sample(&self, _pos: &impl super::NoisePos) -> f64 {
        1f64
    }

    fn fill(&self, densities: &[f64], _applier: &impl super::Applier) -> Vec<f64> {
        densities.iter().map(|_| 1f64).collect()
    }

    fn max(&self) -> f64 {
        1f64
    }

    fn min(&self) -> f64 {
        1f64
    }

    fn apply(&'a self, visitor: &'a impl super::Visitor) -> DensityFunction<'a> {
        visitor.apply(&DensityFunction::BlendAlpha(self.clone()))
    }
}

#[derive(Clone)]
pub struct BlendDensityFunction<'a> {
    function: Arc<DensityFunction<'a>>,
}

impl<'a> BlendDensityFunction<'a> {
    fn apply_density(&self, pos: &impl NoisePos, density: f64) -> f64 {
        pos.get_blender().apply_blend_density(pos, density)
    }
}

impl<'a> DensityFunctionImpl<'a> for BlendDensityFunction<'a> {
    fn sample(&self, pos: &impl super::NoisePos) -> f64 {
        self.apply_density(pos, self.function.sample(pos))
    }

    fn fill(&self, densities: &[f64], applier: &impl super::Applier) -> Vec<f64> {
        let densities = self.function.fill(densities, applier);
        densities
            .iter()
            .enumerate()
            .map(|(i, x)| self.apply_density(&applier.at(i as i32), *x))
            .collect()
    }

    fn apply(&'a self, visitor: &'a impl super::Visitor) -> DensityFunction<'a> {
        let new_function = BlendDensityFunction {
            function: Arc::new(self.function.apply(visitor)),
        };
        visitor.apply(&DensityFunction::BlendDensity(new_function))
    }

    fn min(&self) -> f64 {
        f64::NEG_INFINITY
    }

    fn max(&self) -> f64 {
        f64::INFINITY
    }
}
