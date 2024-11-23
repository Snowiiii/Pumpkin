use pumpkin_core::random::{legacy_rand::LegacyRand, RandomImpl};

use crate::world_gen::noise::simplex::SimplexNoiseSampler;

use super::{
    component_functions::{
        ApplierImpl, ComponentFunctionImpl, ImmutableComponentFunctionImpl,
        SharedConverterEnvironment,
    },
    NoisePosImpl,
};

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

        let f = ((x * x + z * z) as f32).sqrt().mul_add(-8f32, 100f32);
        let mut f = f.clamp(-100f32, 80f32);

        for m in -12..=12 {
            for n in -12..=12 {
                let o = (i + m) as i64;
                let p = (j + n) as i64;

                if (o * o + p * p) > 4096i64
                    && sampler.sample_2d(o as f64, p as f64) < -0.9f32 as f64
                {
                    let g =
                        (o as f32).abs().mul_add(3439f32, (p as f32).abs() * 147f32) % 13f32 + 9f32;
                    let h = (k - m * 2) as f32;
                    let q = (l - n * 2) as f32;
                    let r = h.hypot(q).mul_add(-g, 100f32);
                    let s = r.clamp(-100f32, 80f32);

                    f = f.max(s);
                }
            }
        }

        f
    }
}

impl ComponentFunctionImpl for EndIslandFunction {}

impl ImmutableComponentFunctionImpl for EndIslandFunction {
    fn sample(&self, pos: &super::NoisePos) -> f64 {
        (Self::sample_2d(&self.sampler, pos.x() / 8, pos.z() / 8) as f64 - 8f64) / 128f64
    }

    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        applier.fill(arr, self);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::End
    }
}
