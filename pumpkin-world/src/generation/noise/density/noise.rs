use std::{marker::PhantomData, sync::Arc};

use pumpkin_core::random::{xoroshiro128::Xoroshiro, RandomGenerator, RandomImpl};

use crate::{
    generation::noise::{
        clamped_lerp,
        perlin::{DoublePerlinNoiseParameters, DoublePerlinNoiseSampler, OctavePerlinNoiseSampler},
    },
    match_ref_implementations,
};

use super::{
    component_functions::{
        ApplierImpl, ComponentFunctionImpl, ComponentReference, ComponentReferenceImplementation,
        ConverterEnvironment, ConverterImpl, DensityFunctionEnvironment, EnvironmentApplierImpl,
        ImmutableComponentFunctionImpl, MutableComponentFunctionImpl, MutableComponentReference,
        OwnedConverterEnvironment, SharedComponentReference, SharedConverterEnvironment,
    },
    NoisePos, NoisePosImpl,
};

pub(crate) struct InternalNoise {
    pub(crate) parameters: &'static DoublePerlinNoiseParameters,
    pub(crate) sampler: Option<DoublePerlinNoiseSampler>,
}

impl InternalNoise {
    pub(crate) fn new(
        parameters: &'static DoublePerlinNoiseParameters,
        sampler: Option<DoublePerlinNoiseSampler>,
    ) -> Self {
        Self {
            parameters,
            sampler,
        }
    }

    pub(crate) fn sample(&self, x: f64, y: f64, z: f64) -> f64 {
        match &self.sampler {
            Some(sampler) => sampler.sample(x, y, z),
            None => 0f64,
        }
    }
}

pub struct NoiseFunction {
    pub(crate) noise: Arc<InternalNoise>,
    pub(crate) xz_scale: f64,
    pub(crate) y_scale: f64,
}

impl NoiseFunction {
    pub fn new(noise: Arc<InternalNoise>, xz_scale: f64, y_scale: f64) -> Self {
        Self {
            noise,
            xz_scale,
            y_scale,
        }
    }
}

impl ComponentFunctionImpl for NoiseFunction {}

impl ImmutableComponentFunctionImpl for NoiseFunction {
    #[inline]
    fn sample(&self, pos: &NoisePos) -> f64 {
        self.noise.sample(
            pos.x() as f64 * self.xz_scale,
            pos.y() as f64 * self.y_scale,
            pos.z() as f64 * self.xz_scale,
        )
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        applier.fill(arr, self);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::Noise(self)
    }
}

#[derive(Clone)]
pub(crate) struct ShiftedNoiseUntypedData {
    pub(crate) xz_scale: f64,
    pub(crate) y_scale: f64,
}

pub struct ShiftedNoiseFunction<
    E: DensityFunctionEnvironment,
    R1: ComponentReference<E>,
    R2: ComponentReference<E>,
    R3: ComponentReference<E>,
> {
    pub(crate) shift_x: R1,
    pub(crate) shift_y: R2,
    pub(crate) shift_z: R3,
    pub(crate) noise: Arc<InternalNoise>,
    pub(crate) data: ShiftedNoiseUntypedData,
    _dummy: PhantomData<E>,
}

impl<
        E: DensityFunctionEnvironment,
        R1: ComponentReference<E>,
        R2: ComponentReference<E>,
        R3: ComponentReference<E>,
    > From<ShiftedNoiseFunction<E, R1, R2, R3>> for MutableComponentReference<E>
{
    fn from(value: ShiftedNoiseFunction<E, R1, R2, R3>) -> Self {
        Self(Box::new(value))
    }
}

impl<
        E: DensityFunctionEnvironment,
        R1: ComponentReference<E>,
        R2: ComponentReference<E>,
        R3: ComponentReference<E>,
    > ShiftedNoiseFunction<E, R1, R2, R3>
{
    pub fn new(
        shift_x: R1,
        shift_y: R2,
        shift_z: R3,
        xz_scale: f64,
        y_scale: f64,
        noise: Arc<InternalNoise>,
    ) -> Self {
        Self {
            shift_x,
            shift_y,
            shift_z,
            noise,
            data: ShiftedNoiseUntypedData { xz_scale, y_scale },
            _dummy: PhantomData::<E> {},
        }
    }

    pub fn create_new_ref(
        x: ComponentReferenceImplementation<E>,
        y: ComponentReferenceImplementation<E>,
        z: ComponentReferenceImplementation<E>,
        noise: Arc<InternalNoise>,
        data: &ShiftedNoiseUntypedData,
    ) -> ComponentReferenceImplementation<E> {
        match (x, y, z) {
            (
                ComponentReferenceImplementation::Shared(shared_x),
                ComponentReferenceImplementation::Shared(shared_y),
                ComponentReferenceImplementation::Shared(shared_z),
            ) => ShiftedNoiseFunction::<
                E,
                SharedComponentReference,
                SharedComponentReference,
                SharedComponentReference,
            >::new(
                shared_x,
                shared_y,
                shared_z,
                data.xz_scale,
                data.y_scale,
                noise,
            )
            .into(),
            (x, y, z) => {
                match_ref_implementations!(
                    (ref_x, x),
                    (ref_y, y),
                    (ref_z, z);
                    {
                        ComponentReferenceImplementation::Mutable(
                            MutableComponentReference(Box::new(
                                ShiftedNoiseFunction::new(
                                    ref_x,
                                    ref_y,
                                    ref_z,
                                    data.xz_scale,
                                    data.y_scale,
                                    noise
                                )
                            ))
                        )
                    }
                )
            }
        }
    }
}

impl<
        E: DensityFunctionEnvironment,
        R1: ComponentReference<E>,
        R2: ComponentReference<E>,
        R3: ComponentReference<E>,
    > ComponentFunctionImpl for ShiftedNoiseFunction<E, R1, R2, R3>
{
}

impl<
        E: DensityFunctionEnvironment,
        R1: ComponentReference<E>,
        R2: ComponentReference<E>,
        R3: ComponentReference<E>,
    > MutableComponentFunctionImpl<E> for ShiftedNoiseFunction<E, R1, R2, R3>
{
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f64 {
        let x_pos = (pos.x() as f64) * self.data.xz_scale + self.shift_x.sample_mut(pos, env);
        let y_pos = (pos.y() as f64) * self.data.y_scale + self.shift_y.sample_mut(pos, env);
        let z_pos = (pos.z() as f64) * self.data.xz_scale + self.shift_z.sample_mut(pos, env);

        self.noise.sample(x_pos, y_pos, z_pos)
    }

    #[inline]
    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>) {
        applier.fill_mut(arr, self);
    }

    fn environment(&self) -> ConverterEnvironment<E> {
        ConverterEnvironment::ShiftedNoise(
            &self.shift_x,
            &self.shift_y,
            &self.shift_z,
            &self.noise,
            &self.data,
        )
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<E> {
        OwnedConverterEnvironment::ShiftedNoise(
            self.shift_x.wrapped_ref(),
            self.shift_y.wrapped_ref(),
            self.shift_z.wrapped_ref(),
            self.noise,
            self.data,
        )
    }

    fn convert(
        self: Box<Self>,
        converter: &mut dyn ConverterImpl<E>,
    ) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(
            self.shift_x.convert(converter),
            self.shift_y.convert(converter),
            self.shift_z.convert(converter),
            converter
                .convert_noise(&self.noise)
                .unwrap_or_else(|| self.noise.clone()),
            &self.data,
        )
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(
            self.shift_x.clone_to_new_ref(),
            self.shift_y.clone_to_new_ref(),
            self.shift_z.clone_to_new_ref(),
            self.noise.clone(),
            &self.data,
        )
    }
}

impl<E: DensityFunctionEnvironment> ImmutableComponentFunctionImpl
    for ShiftedNoiseFunction<
        E,
        SharedComponentReference,
        SharedComponentReference,
        SharedComponentReference,
    >
{
    fn sample(&self, pos: &NoisePos) -> f64 {
        let x_pos = (pos.x() as f64) * self.data.xz_scale + self.shift_x.sample(pos);
        let y_pos = (pos.y() as f64) * self.data.y_scale + self.shift_y.sample(pos);
        let z_pos = (pos.z() as f64) * self.data.xz_scale + self.shift_z.sample(pos);

        self.noise.sample(x_pos, y_pos, z_pos)
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        applier.fill(arr, self);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::ShiftedNoise(
            &self.shift_x,
            &self.shift_y,
            &self.shift_z,
            &self.noise,
            &self.data,
        )
    }
}

pub struct InterpolatedNoiseFunction {
    pub(crate) lower: Arc<OctavePerlinNoiseSampler>,
    pub(crate) upper: Arc<OctavePerlinNoiseSampler>,
    pub(crate) interpolation: Arc<OctavePerlinNoiseSampler>,
    pub(crate) xz_scale_scaled: f64,
    pub(crate) y_scale_scaled: f64,
    pub(crate) xz_factor: f64,
    pub(crate) y_factor: f64,
    pub(crate) smear_scale: f64,
    pub(crate) xz_scale: f64,
    pub(crate) y_scale: f64,
}

impl InterpolatedNoiseFunction {
    fn create_from_random(
        rand: &mut RandomGenerator,
        xz_scale: f64,
        y_scale: f64,
        xz_factor: f64,
        y_factor: f64,
        smear_scale: f64,
    ) -> Self {
        // TODO: This can be const
        let (start_1, amplitudes_1) =
            OctavePerlinNoiseSampler::calculate_amplitudes(&(-15..=0).collect::<Vec<i32>>());

        let (start_2, amplitudes_2) =
            OctavePerlinNoiseSampler::calculate_amplitudes(&(-7..=0).collect::<Vec<i32>>());

        Self::new(
            OctavePerlinNoiseSampler::new(rand, start_1, &amplitudes_1, true),
            OctavePerlinNoiseSampler::new(rand, start_1, &amplitudes_1, true),
            OctavePerlinNoiseSampler::new(rand, start_2, &amplitudes_2, true),
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
        let mut rand = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(0));
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
        }
    }
}

impl ComponentFunctionImpl for InterpolatedNoiseFunction {}

impl ImmutableComponentFunctionImpl for InterpolatedNoiseFunction {
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

    #[inline]
    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        applier.fill(arr, self);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::InterpolatedNoise(self)
    }
}
