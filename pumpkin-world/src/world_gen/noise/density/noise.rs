use std::sync::Arc;

use pumpkin_core::random::{xoroshiro128::Xoroshiro, RandomGenerator, RandomImpl};

use crate::world_gen::noise::{
    clamped_lerp,
    perlin::{DoublePerlinNoiseParameters, DoublePerlinNoiseSampler, OctavePerlinNoiseSampler},
};

use super::{
    Applier, ApplierImpl, DensityFunction, DensityFunctionImpl, NoisePos, NoisePosImpl, Visitor,
    VisitorImpl,
};

pub(crate) struct InternalNoise<'a> {
    data: DoublePerlinNoiseParameters<'a>,
    sampler: Option<DoublePerlinNoiseSampler>,
}

impl<'a> InternalNoise<'a> {
    pub(crate) fn new(
        data: DoublePerlinNoiseParameters<'a>,
        function: Option<DoublePerlinNoiseSampler>,
    ) -> Self {
        Self {
            data,
            sampler: function,
        }
    }

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
    pub(crate) noise: Arc<InternalNoise<'a>>,
    xz_scale: f64,
    y_scale: f64,
}

impl<'a> NoiseFunction<'a> {
    pub fn new(noise: Arc<InternalNoise<'a>>, xz_scale: f64, y_scale: f64) -> Self {
        Self {
            noise,
            xz_scale,
            y_scale,
        }
    }
}

impl<'a> DensityFunctionImpl<'a> for NoiseFunction<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        self.noise.sample(
            pos.x() as f64 * self.xz_scale,
            pos.y() as f64 * self.y_scale,
            pos.z() as f64 * self.xz_scale,
        )
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
        applier.fill(densities, &DensityFunction::Noise(self.clone()))
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::Noise(self.clone())))
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
    shift_x: Arc<DensityFunction<'a>>,
    shift_y: Arc<DensityFunction<'a>>,
    shift_z: Arc<DensityFunction<'a>>,
    noise: Arc<InternalNoise<'a>>,
    xz_scale: f64,
    y_scale: f64,
}

impl<'a> ShiftedNoiseFunction<'a> {
    pub fn new(
        shift_x: Arc<DensityFunction<'a>>,
        shift_y: Arc<DensityFunction<'a>>,
        shift_z: Arc<DensityFunction<'a>>,
        xz_scale: f64,
        y_scale: f64,
        noise: Arc<InternalNoise<'a>>,
    ) -> Self {
        Self {
            shift_x,
            shift_y,
            shift_z,
            noise,
            xz_scale,
            y_scale,
        }
    }
}

impl<'a> DensityFunctionImpl<'a> for ShiftedNoiseFunction<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        let d = (pos.x() as f64).mul_add(self.xz_scale, self.shift_x.sample(pos));
        let e = (pos.y() as f64).mul_add(self.y_scale, self.shift_y.sample(pos));
        let f = (pos.z() as f64).mul_add(self.xz_scale, self.shift_z.sample(pos));

        self.noise.sample(d, e, f)
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
        applier.fill(densities, &DensityFunction::ShiftedNoise(self.clone()))
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        let new_x = self.shift_x.apply(visitor);
        let new_y = self.shift_y.apply(visitor);
        let new_z = self.shift_z.apply(visitor);
        let new_noise = visitor.apply_internal_noise(self.noise.clone());

        Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction {
            shift_x: new_x,
            shift_y: new_y,
            shift_z: new_z,
            xz_scale: self.xz_scale,
            y_scale: self.y_scale,
            noise: new_noise,
        }))
    }

    fn max(&self) -> f64 {
        self.noise.max_value()
    }

    fn min(&self) -> f64 {
        -self.max()
    }
}

#[derive(Clone)]
pub struct InterpolatedNoiseSampler {
    lower: Arc<OctavePerlinNoiseSampler>,
    upper: Arc<OctavePerlinNoiseSampler>,
    interpolation: Arc<OctavePerlinNoiseSampler>,
    xz_scale_scaled: f64,
    y_scale_scaled: f64,
    xz_factor: f64,
    y_factor: f64,
    smear_scale: f64,
    max_value: f64,
    xz_scale: f64,
    y_scale: f64,
}

impl InterpolatedNoiseSampler {
    fn create_from_random(
        rand: &mut RandomGenerator,
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale: f64,
    ) -> Self {
        let (start_1, amplitudes_1) =
            OctavePerlinNoiseSampler::calculate_amplitudes(&(-15..=0).collect::<Vec<i32>>());

        let (start_2, amplitudes_2) =
            OctavePerlinNoiseSampler::calculate_amplitudes(&(-7..=0).collect::<Vec<i32>>());

        Self::new(
            OctavePerlinNoiseSampler::new(rand, start_1, &amplitudes_1),
            OctavePerlinNoiseSampler::new(rand, start_1, &amplitudes_1),
            OctavePerlinNoiseSampler::new(rand, start_2, &amplitudes_2),
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale,
        )
    }

    pub fn copy_with_random(&self, rand: &mut RandomGenerator) -> Self {
        Self::create_from_random(
            rand,
            self.xz_scale,
            self.y_scale,
            self.xz_factor,
            self.y_factor,
            self.smear_scale,
        )
    }

    pub fn create_base_3d_noise_function(
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale: f64,
    ) -> Self {
        let mut rand = RandomGenerator::LegacyXoroshiro(Xoroshiro::from_seed(0));
        Self::create_from_random(
            &mut rand,
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new(
        lower: OctavePerlinNoiseSampler,
        upper: OctavePerlinNoiseSampler,
        interpolation: OctavePerlinNoiseSampler,
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale: f64,
    ) -> Self {
        let y_scale_scaled = 684.412f64 * y_scale;
        let max_value = OctavePerlinNoiseSampler::get_total_amplitude(
            y_scale_scaled + 2f64,
            lower.persistence,
            &lower.amplitudes,
        );
        Self {
            lower: Arc::new(lower),
            upper: Arc::new(upper),
            interpolation: Arc::new(interpolation),
            xz_scale,
            y_scale,
            xz_factor,
            y_factor,
            smear_scale,
            y_scale_scaled,
            xz_scale_scaled: 684.412f64 * xz_scale,
            max_value,
        }
    }
}

impl<'a> DensityFunctionImpl<'a> for InterpolatedNoiseSampler {
    fn sample(&self, pos: &NoisePos) -> f64 {
        let d = pos.x() as f64 * self.xz_scale_scaled;
        let e = pos.y() as f64 * self.y_scale_scaled;
        let f = pos.z() as f64 * self.xz_scale_scaled;

        let g = d / self.xz_factor;
        let h = e / self.y_factor;
        let i = f / self.xz_factor;

        let j = self.y_scale_scaled * self.smear_scale;
        let k = j / self.y_factor;

        let mut n = 0f64;
        let mut o = 1f64;

        for p in 0..8 {
            let sampler = self.interpolation.get_octave(p);
            if let Some(sampler) = sampler {
                n += sampler.sample_no_fade(
                    OctavePerlinNoiseSampler::maintain_precision(g * o),
                    OctavePerlinNoiseSampler::maintain_precision(h * o),
                    OctavePerlinNoiseSampler::maintain_precision(i * o),
                    k * o,
                    h * o,
                ) / o;
            }

            o /= 2f64;
        }

        let q = (n / 10f64 + 1f64) / 2f64;
        let bl2 = q >= 1f64;
        let bl3 = q <= 0f64;
        let mut o = 1f64;
        let mut l = 0f64;
        let mut m = 0f64;

        for r in 0..16 {
            let s = OctavePerlinNoiseSampler::maintain_precision(d * o);
            let t = OctavePerlinNoiseSampler::maintain_precision(e * o);
            let u = OctavePerlinNoiseSampler::maintain_precision(f * o);
            let v = j * o;

            if !bl2 {
                let sampler = self.lower.get_octave(r);
                if let Some(sampler) = sampler {
                    l += sampler.sample_no_fade(s, t, u, v, e * o) / o;
                }
            }

            if !bl3 {
                let sampler = self.upper.get_octave(r);
                if let Some(sampler) = sampler {
                    m += sampler.sample_no_fade(s, t, u, v, e * o) / o;
                }
            }

            o /= 2f64;
        }

        clamped_lerp(l / 512f64, m / 512f64, q) / 128f64
    }

    fn max(&self) -> f64 {
        self.max_value
    }

    fn min(&self) -> f64 {
        -self.max()
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier) {
        applier.fill(densities, &DensityFunction::InterpolatedNoise(self.clone()))
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::InterpolatedNoise(self.clone())))
    }
}
