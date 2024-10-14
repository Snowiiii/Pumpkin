use std::sync::Arc;

use super::{
    Applier, DensityFunction, DensityFunctionImpl, NoisePos, UnaryDensityFunction, Visitor,
};

#[derive(Clone)]
pub struct ClampFunction<'a> {
    pub(crate) input: Arc<DensityFunction<'a>>,
    pub(crate) min: f64,
    pub(crate) max: f64,
}

impl<'a> UnaryDensityFunction<'a> for ClampFunction<'a> {
    fn apply_density(&self, density: f64) -> f64 {
        density.clamp(self.min, self.max)
    }
}

impl<'a> DensityFunctionImpl<'a> for ClampFunction<'a> {
    fn apply(&'a self, visitor: &'a Visitor) -> Arc<DensityFunction<'a>> {
        Arc::new(DensityFunction::Clamp(ClampFunction {
            input: self.input.apply(visitor),
            min: self.min,
            max: self.max,
        }))
    }

    fn sample(&self, pos: &NoisePos) -> f64 {
        self.apply_density(self.input.sample(pos))
    }

    fn fill(&self, densities: &[f64], applier: &Applier) -> Vec<f64> {
        let densities = self.input.fill(densities, applier);
        densities.iter().map(|x| self.apply_density(*x)).collect()
    }

    fn min(&self) -> f64 {
        self.min
    }

    fn max(&self) -> f64 {
        self.max
    }
}

#[derive(Clone)]
pub(crate) enum UnaryType {
    Abs,
    Square,
    Cube,
    HalfNeg,
    QuartNeg,
    Squeeze,
}

#[derive(Clone)]
pub struct UnaryFunction<'a> {
    action: UnaryType,
    input: Arc<DensityFunction<'a>>,
    min: f64,
    max: f64,
}

impl<'a> UnaryFunction<'a> {
    pub(crate) fn create(action: UnaryType, input: Arc<DensityFunction<'a>>) -> UnaryFunction {
        let base_min = input.min();
        let new_min = Self::internal_apply(&action, base_min);
        let new_max = Self::internal_apply(&action, input.max());
        match action {
            UnaryType::Abs | UnaryType::Square => Self {
                action,
                input,
                min: f64::max(0f64, base_min),
                max: f64::max(new_min, new_max),
            },
            _ => Self {
                action,
                input,
                min: new_min,
                max: new_max,
            },
        }
    }

    fn internal_apply(action: &UnaryType, density: f64) -> f64 {
        match action {
            UnaryType::Abs => density.abs(),
            UnaryType::Square => density * density,
            UnaryType::Cube => density * density * density,
            UnaryType::HalfNeg => {
                if density > 0f64 {
                    density
                } else {
                    density * 0.5f64
                }
            }
            UnaryType::QuartNeg => {
                if density > 0f64 {
                    density
                } else {
                    density * 0.25f64
                }
            }
            UnaryType::Squeeze => {
                let d = density.clamp(-1f64, 1f64);
                d / 2f64 - d * d * d / 24f64
            }
        }
    }
}

impl<'a> UnaryDensityFunction<'a> for UnaryFunction<'a> {
    fn apply_density(&self, density: f64) -> f64 {
        Self::internal_apply(&self.action, density)
    }
}

impl<'a> DensityFunctionImpl<'a> for UnaryFunction<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        self.apply_density(self.input.sample(pos))
    }

    fn fill(&self, densities: &[f64], applier: &Applier) -> Vec<f64> {
        let densities = self.input.fill(densities, applier);
        densities.iter().map(|x| self.apply_density(*x)).collect()
    }

    fn apply(&'a self, visitor: &'a Visitor) -> Arc<DensityFunction<'a>> {
        let raw = Self::create(self.action.clone(), self.input.apply(visitor));
        Arc::new(DensityFunction::Unary(raw))
    }

    fn max(&self) -> f64 {
        self.max
    }

    fn min(&self) -> f64 {
        self.min
    }
}
