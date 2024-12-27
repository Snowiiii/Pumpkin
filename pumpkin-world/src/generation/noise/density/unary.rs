use std::marker::PhantomData;

use super::{
    component_functions::{
        ApplierImpl, ComponentFunctionImpl, ComponentReference, ComponentReferenceImplementation,
        ConverterEnvironment, ConverterImpl, DensityFunctionEnvironment, EnvironmentApplierImpl,
        ImmutableComponentFunctionImpl, MutableComponentFunctionImpl, MutableComponentReference,
        NoEnvironment, OwnedConverterEnvironment, SharedComponentReference,
        SharedConverterEnvironment,
    },
    NoisePos,
};

#[derive(Clone)]
pub(crate) struct ClampUntypedData {
    pub(crate) min: f64,
    pub(crate) max: f64,
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> From<ClampFunction<E, R>>
    for MutableComponentReference<E>
{
    fn from(value: ClampFunction<E, R>) -> Self {
        Self(Box::new(value))
    }
}

pub struct ClampFunction<E: DensityFunctionEnvironment, R: ComponentReference<E>> {
    pub(crate) input: R,
    pub(crate) data: ClampUntypedData,
    _dummy: PhantomData<E>,
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> ClampFunction<E, R> {
    pub fn new(input: R, min: f64, max: f64) -> Self {
        Self {
            input,
            data: ClampUntypedData { min, max },
            _dummy: PhantomData::<E> {},
        }
    }
    #[inline]
    fn apply_density(&self, density: f64) -> f64 {
        density.clamp(self.data.min, self.data.max)
    }

    pub fn create_new_ref(
        input: ComponentReferenceImplementation<E>,
        data: &ClampUntypedData,
    ) -> ComponentReferenceImplementation<E> {
        match input {
            ComponentReferenceImplementation::Shared(shared) => {
                ComponentReferenceImplementation::Shared(
                    ClampFunction::<NoEnvironment, SharedComponentReference>::new(
                        shared, data.min, data.max,
                    )
                    .into(),
                )
            }
            ComponentReferenceImplementation::Mutable(owned) => {
                ComponentReferenceImplementation::Mutable(
                    ClampFunction::new(owned, data.min, data.max).into(),
                )
            }
        }
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> ComponentFunctionImpl
    for ClampFunction<E, R>
{
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> MutableComponentFunctionImpl<E>
    for ClampFunction<E, R>
{
    #[inline]
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f64 {
        let density = self.input.sample_mut(pos, env);
        self.apply_density(density)
    }

    #[inline]
    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>) {
        self.input.fill_mut(arr, applier);
        arr.iter_mut()
            .for_each(|density| *density = self.apply_density(*density));
    }

    fn environment(&self) -> ConverterEnvironment<E> {
        ConverterEnvironment::Clamp(&self.input, &self.data)
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<E> {
        OwnedConverterEnvironment::Clamp(self.input.wrapped_ref(), self.data)
    }

    fn convert(
        self: Box<Self>,
        converter: &mut dyn ConverterImpl<E>,
    ) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(self.input.convert(converter), &self.data)
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(self.input.clone_to_new_ref(), &self.data)
    }
}

impl<E: DensityFunctionEnvironment> ImmutableComponentFunctionImpl
    for ClampFunction<E, SharedComponentReference>
{
    #[inline]
    fn sample(&self, pos: &NoisePos) -> f64 {
        let density = self.input.sample(pos);
        self.apply_density(density)
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        self.input.fill(arr, applier);
        arr.iter_mut()
            .for_each(|density| *density = self.apply_density(*density));
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::Clamp(&self.input, &self.data)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum UnaryType {
    Abs,
    Square,
    Cube,
    HalfNeg,
    QuartNeg,
    Squeeze,
}

pub struct UnaryFunction<E: DensityFunctionEnvironment, R: ComponentReference<E>> {
    pub(crate) unary_type: UnaryType,
    pub(crate) input: R,
    _dummy: PhantomData<E>,
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> From<UnaryFunction<E, R>>
    for MutableComponentReference<E>
{
    fn from(value: UnaryFunction<E, R>) -> Self {
        Self(Box::new(value))
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> UnaryFunction<E, R> {
    pub fn new(unary_type: UnaryType, input: R) -> Self {
        Self {
            unary_type,
            input,
            _dummy: PhantomData::<E> {},
        }
    }

    #[inline]
    fn apply_density(&self, density: f64) -> f64 {
        match self.unary_type {
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

    pub fn create_new_ref(
        input: ComponentReferenceImplementation<E>,
        action: UnaryType,
    ) -> ComponentReferenceImplementation<E> {
        match input {
            ComponentReferenceImplementation::Shared(shared) => {
                ComponentReferenceImplementation::Shared(
                    UnaryFunction::<NoEnvironment, SharedComponentReference>::new(action, shared)
                        .into(),
                )
            }
            ComponentReferenceImplementation::Mutable(owned) => {
                ComponentReferenceImplementation::Mutable(UnaryFunction::new(action, owned).into())
            }
        }
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> ComponentFunctionImpl
    for UnaryFunction<E, R>
{
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> MutableComponentFunctionImpl<E>
    for UnaryFunction<E, R>
{
    #[inline]
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f64 {
        let density = self.input.sample_mut(pos, env);
        self.apply_density(density)
    }

    #[inline]
    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>) {
        self.input.fill_mut(arr, applier);
        arr.iter_mut()
            .for_each(|density| *density = self.apply_density(*density));
    }

    fn environment(&self) -> ConverterEnvironment<E> {
        ConverterEnvironment::Unary(&self.input, self.unary_type)
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<E> {
        OwnedConverterEnvironment::Unary(self.input.wrapped_ref(), self.unary_type)
    }

    fn convert(
        self: Box<Self>,
        converter: &mut dyn ConverterImpl<E>,
    ) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(self.input.convert(converter), self.unary_type)
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(self.input.clone_to_new_ref(), self.unary_type)
    }
}

impl<E: DensityFunctionEnvironment> ImmutableComponentFunctionImpl
    for UnaryFunction<E, SharedComponentReference>
{
    #[inline]
    fn sample(&self, pos: &NoisePos) -> f64 {
        let density = self.input.sample(pos);
        self.apply_density(density)
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        self.input.fill(arr, applier);
        arr.iter_mut()
            .for_each(|density| *density = self.apply_density(*density));
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::Unary(&self.input, self.unary_type)
    }
}
