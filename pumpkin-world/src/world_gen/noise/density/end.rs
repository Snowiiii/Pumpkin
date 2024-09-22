use std::sync::Arc;

use pumpkin_core::random::{legacy_rand::LegacyRand, RandomImpl};

use crate::world_gen::noise::simplex::SimplexNoiseSampler;

use super::{
    Applier, ApplierImpl, DensityFunction, DensityFunctionImpl, NoisePos, NoisePosImpl, Visitor,
    VisitorImpl,
};

#[derive(Clone)]
pub struct EndIslandFunction {
    sampler: SimplexNoiseSampler,
}

impl EndIslandFunction {
    pub fn new(seed: u64) -> Self {
        let mut rand = LegacyRand::from_seed(seed);
        rand.skip(17292);
        Self {
            sampler: SimplexNoiseSampler::new(&mut rand),
        }
    }

    fn sample_2d(sampler: &SimplexNoiseSampler, x: i32, z: i32) -> f32 {
        let i = x / 2;
        let j = z / 2;
        let k = x % 2;
        let l = z % 2;

        let f = 100f32 - ((x * x + z * z) as f32).sqrt() * 8f32;
        let mut f = f.clamp(-100f32, 80f32);

        for m in -12..=12 {
            for n in -12..=12 {
                let o = (i + m) as i64;
                let p = (j + n) as i64;

                if (o * o + p * p) > 4096i64
                    && sampler.sample_2d(o as f64, p as f64) < -0.9f32 as f64
                {
                    let g = ((o as f32).abs() * 3439f32 + (p as f32).abs() * 147f32) % 13f32 + 9f32;
                    let h = (k - m * 2) as f32;
                    let q = (l - n * 2) as f32;
                    let r = 100f32 - (h * h + q * q).sqrt() * g;
                    let s = r.clamp(-100f32, 80f32);

                    f = f.max(s);
                }
            }
        }

        f
    }
}

impl<'a> DensityFunctionImpl<'a> for EndIslandFunction {
    fn fill(&self, densities: &[f64], applier: &Applier) -> Vec<f64> {
        applier.fill(densities, &DensityFunction::EndIsland(self.clone()))
    }

    fn sample(&self, pos: &NoisePos) -> f64 {
        (Self::sample_2d(&self.sampler, pos.x() / 8, pos.z() / 8) as f64 - 8f64) / 128f64
    }

    fn min(&self) -> f64 {
        -0.84375f64
    }

    fn max(&self) -> f64 {
        0.5625f64
    }

    fn apply(&'a self, visitor: &'a Visitor) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::EndIsland(self.clone())))
    }
}
