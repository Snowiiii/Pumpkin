use std::sync::Arc;

use super::{
    noise::InternalNoise, Applier, ApplierImpl, DensityFunction, DensityFunctionImpl, NoisePos,
    NoisePosImpl, Visitor, VisitorImpl,
};

#[derive(Clone)]
pub enum RarityMapper {
    Tunnels,
    Caves,
}

impl RarityMapper {
    #[inline]
    pub fn max_multiplier(&self) -> f64 {
        match self {
            Self::Tunnels => 2f64,
            Self::Caves => 3f64,
        }
    }

    #[inline]
    pub fn scale(&self, value: f64) -> f64 {
        match self {
            Self::Tunnels => {
                if value < -0.5f64 {
                    0.75f64
                } else if value < 0f64 {
                    1f64
                } else if value < 0.5f64 {
                    1.5f64
                } else {
                    2f64
                }
            }
            Self::Caves => {
                if value < -0.75f64 {
                    0.5f64
                } else if value < -0.5f64 {
                    0.75f64
                } else if value < 0.5f64 {
                    1f64
                } else if value < 0.75 {
                    2f64
                } else {
                    3f64
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct WierdScaledFunction<'a> {
    input: Arc<DensityFunction<'a>>,
    noise: Arc<InternalNoise<'a>>,
    rarity: RarityMapper,
}

impl<'a> WierdScaledFunction<'a> {
    pub fn new(
        input: Arc<DensityFunction<'a>>,
        noise: Arc<InternalNoise<'a>>,
        rarity: RarityMapper,
    ) -> Self {
        Self {
            input,
            noise,
            rarity,
        }
    }

    fn apply_loc(&self, pos: &NoisePos, density: f64) -> f64 {
        let d = self.rarity.scale(density);
        d * self
            .noise
            .sample(pos.x() as f64 / d, pos.y() as f64 / d, pos.z() as f64 / d)
            .abs()
    }
}

impl<'a> DensityFunctionImpl<'a> for WierdScaledFunction<'a> {
    fn max(&self) -> f64 {
        self.rarity.max_multiplier() * self.noise.max_value()
    }

    fn min(&self) -> f64 {
        0f64
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::Wierd(WierdScaledFunction {
            input: self.input.apply(visitor),
            noise: visitor.apply_internal_noise(self.noise.clone()),
            rarity: self.rarity.clone(),
        })))
    }

    fn sample(&self, pos: &NoisePos) -> f64 {
        self.apply_loc(pos, self.input.sample(pos))
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
        self.input.fill(densities, applier);
        densities.iter_mut().enumerate().for_each(|(i, val)| {
            *val = self.apply_loc(&applier.at(i), *val);
        });
    }
}
