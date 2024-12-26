use std::marker::PhantomData;

use crate::generation::blender::BlenderImpl;

use super::{
    component_functions::{
        ApplierImpl, ComponentFunctionImpl, ComponentReference, ComponentReferenceImplementation,
        ConverterEnvironment, ConverterImpl, DensityFunctionEnvironment, EnvironmentApplierImpl,
        ImmutableComponentFunctionImpl, MutableComponentFunctionImpl, MutableComponentReference,
        NoEnvironment, OwnedConverterEnvironment, SharedComponentReference,
        SharedConverterEnvironment,
    },
    NoisePos, NoisePosImpl,
};

pub struct BlendOffsetFunction {}

impl BlendOffsetFunction {
    pub const INSTANCE: Self = Self {};
}

impl ComponentFunctionImpl for BlendOffsetFunction {}

impl ImmutableComponentFunctionImpl for BlendOffsetFunction {
    #[inline]
    fn sample(&self, _pos: &NoisePos) -> f64 {
        0f64
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], _applier: &mut dyn ApplierImpl) {
        arr.fill(0f64);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::BlendOffset
    }
}

pub struct BlendAlphaFunction {}

impl BlendAlphaFunction {
    pub const INSTANCE: Self = Self {};
}

impl ComponentFunctionImpl for BlendAlphaFunction {}

impl ImmutableComponentFunctionImpl for BlendAlphaFunction {
    #[inline]
    fn sample(&self, _pos: &NoisePos) -> f64 {
        1f64
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], _applier: &mut dyn ApplierImpl) {
        arr.fill(1f64);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::BlendAlpha
    }
}

pub struct BlendDensityFunction<E: DensityFunctionEnvironment, R: ComponentReference<E>> {
    pub(crate) function: R,
    _dummy: PhantomData<E>,
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> From<BlendDensityFunction<E, R>>
    for MutableComponentReference<E>
{
    fn from(value: BlendDensityFunction<E, R>) -> Self {
        Self(Box::new(value))
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> BlendDensityFunction<E, R> {
    pub fn new(function: R) -> Self {
        Self {
            function,
            _dummy: PhantomData::<E> {},
        }
    }

    #[inline]
    fn apply_density(&self, pos: &NoisePos, density: f64) -> f64 {
        pos.get_blender().apply_blend_density(pos, density)
    }

    pub fn create_new_ref(
        function: ComponentReferenceImplementation<E>,
    ) -> ComponentReferenceImplementation<E> {
        match function {
            ComponentReferenceImplementation::Shared(shared) => {
                ComponentReferenceImplementation::Shared(
                    BlendDensityFunction::<NoEnvironment, SharedComponentReference>::new(shared)
                        .into(),
                )
            }
            ComponentReferenceImplementation::Mutable(owned) => {
                ComponentReferenceImplementation::Mutable(BlendDensityFunction::new(owned).into())
            }
        }
    }
}
impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> ComponentFunctionImpl
    for BlendDensityFunction<E, R>
{
}

impl<E: DensityFunctionEnvironment> ImmutableComponentFunctionImpl
    for BlendDensityFunction<E, SharedComponentReference>
{
    #[inline]
    fn sample(&self, pos: &NoisePos) -> f64 {
        let density = self.function.sample(pos);
        self.apply_density(pos, density)
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        self.function.fill(arr, applier);
        arr.iter_mut().enumerate().for_each(|(i, x)| {
            *x = self.apply_density(&applier.at(i), *x);
        });
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::BlendDensity(&self.function)
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> MutableComponentFunctionImpl<E>
    for BlendDensityFunction<E, R>
{
    #[inline]
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f64 {
        let density = self.function.sample_mut(pos, env);
        self.apply_density(pos, density)
    }

    #[inline]
    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>) {
        self.function.fill_mut(arr, applier);
        arr.iter_mut().enumerate().for_each(|(i, x)| {
            *x = self.apply_density(&applier.at(i), *x);
        });
    }

    fn environment(&self) -> ConverterEnvironment<E> {
        ConverterEnvironment::BlendDensity(&self.function)
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<E> {
        OwnedConverterEnvironment::BlendDensity(self.function.wrapped_ref())
    }

    fn convert(
        self: Box<Self>,
        converter: &mut dyn ConverterImpl<E>,
    ) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(self.function.convert(converter))
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(self.function.clone_to_new_ref())
    }
}
