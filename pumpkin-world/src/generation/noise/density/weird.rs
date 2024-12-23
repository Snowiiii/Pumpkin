use std::{hash::Hash, marker::PhantomData, sync::Arc};

use super::{
    component_functions::{
        ApplierImpl, ComponentFunctionImpl, ComponentReference, ComponentReferenceImplementation,
        ConverterEnvironment, ConverterImpl, DensityFunctionEnvironment, EnvironmentApplierImpl,
        ImmutableComponentFunctionImpl, MutableComponentFunctionImpl, MutableComponentReference,
        NoEnvironment, OwnedConverterEnvironment, SharedComponentReference,
        SharedConverterEnvironment,
    },
    noise::InternalNoise,
    NoisePos, NoisePosImpl,
};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
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
                } else if value < 0.75f64 {
                    2f64
                } else {
                    3f64
                }
            }
        }
    }
}

pub struct WierdScaledFunction<E: DensityFunctionEnvironment, R: ComponentReference<E>> {
    pub(crate) input: R,
    pub(crate) noise: Arc<InternalNoise>,
    pub(crate) rarity: RarityMapper,
    _dummy: PhantomData<E>,
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> From<WierdScaledFunction<E, R>>
    for MutableComponentReference<E>
{
    fn from(value: WierdScaledFunction<E, R>) -> Self {
        Self(Box::new(value))
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> WierdScaledFunction<E, R> {
    pub fn new(input: R, noise: Arc<InternalNoise>, rarity: RarityMapper) -> Self {
        Self {
            input,
            noise,
            rarity,
            _dummy: PhantomData::<E> {},
        }
    }

    fn apply_loc(&self, pos: &NoisePos, density: f64) -> f64 {
        let d = self.rarity.scale(density);
        d * self
            .noise
            .sample(pos.x() as f64 / d, pos.y() as f64 / d, pos.z() as f64 / d)
            .abs()
    }

    pub fn create_new_ref(
        input: ComponentReferenceImplementation<E>,
        noise: Arc<InternalNoise>,
        rarity: RarityMapper,
    ) -> ComponentReferenceImplementation<E> {
        match input {
            ComponentReferenceImplementation::Mutable(owned) => {
                ComponentReferenceImplementation::Mutable(
                    WierdScaledFunction::new(owned, noise, rarity).into(),
                )
            }
            ComponentReferenceImplementation::Shared(shared) => {
                ComponentReferenceImplementation::Shared(
                    WierdScaledFunction::<NoEnvironment, SharedComponentReference>::new(
                        shared, noise, rarity,
                    )
                    .into(),
                )
            }
        }
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> ComponentFunctionImpl
    for WierdScaledFunction<E, R>
{
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> MutableComponentFunctionImpl<E>
    for WierdScaledFunction<E, R>
{
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f64 {
        let density = self.input.sample_mut(pos, env);
        self.apply_loc(pos, density)
    }

    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>) {
        self.input.fill_mut(arr, applier);
        arr.iter_mut().enumerate().for_each(|(i, val)| {
            *val = self.apply_loc(&applier.at(i), *val);
        });
    }

    fn environment(&self) -> ConverterEnvironment<E> {
        ConverterEnvironment::Wierd(&self.input, &self.noise, self.rarity)
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<E> {
        OwnedConverterEnvironment::Wierd(self.input.wrapped_ref(), self.noise, self.rarity)
    }

    fn convert(
        self: Box<Self>,
        converter: &mut dyn ConverterImpl<E>,
    ) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(
            self.input.convert(converter),
            converter
                .convert_noise(&self.noise)
                .unwrap_or_else(|| self.noise.clone()),
            self.rarity,
        )
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(
            self.input.clone_to_new_ref(),
            self.noise.clone(),
            self.rarity,
        )
    }
}

impl<E: DensityFunctionEnvironment> ImmutableComponentFunctionImpl
    for WierdScaledFunction<E, SharedComponentReference>
{
    fn sample(&self, pos: &NoisePos) -> f64 {
        let density = self.input.sample(pos);
        self.apply_loc(pos, density)
    }

    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        self.input.fill(arr, applier);
        arr.iter_mut().enumerate().for_each(|(i, val)| {
            *val = self.apply_loc(&applier.at(i), *val);
        });
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::Wierd(&self.input, &self.noise, self.rarity)
    }
}
