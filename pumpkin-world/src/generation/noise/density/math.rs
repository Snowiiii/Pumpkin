use std::marker::PhantomData;

use crate::match_ref_implementations;

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

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum LinearType {
    Mul,
    Add,
}

#[derive(Clone)]
pub(crate) struct LinearUntypedData {
    pub(crate) arg: f64,
    pub(crate) action: LinearType,
}

pub struct LinearFunction<E: DensityFunctionEnvironment, R: ComponentReference<E>> {
    pub(crate) input: R,
    pub(crate) data: LinearUntypedData,
    _dummy: PhantomData<E>,
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> From<LinearFunction<E, R>>
    for MutableComponentReference<E>
{
    fn from(value: LinearFunction<E, R>) -> Self {
        Self(Box::new(value))
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> LinearFunction<E, R> {
    pub fn new(action: LinearType, input: R, arg: f64) -> Self {
        Self {
            input,
            data: LinearUntypedData { action, arg },
            _dummy: PhantomData::<E> {},
        }
    }
    #[inline]
    fn apply_density(&self, density: f64) -> f64 {
        match self.data.action {
            LinearType::Mul => density * self.data.arg,
            LinearType::Add => density + self.data.arg,
        }
    }

    #[inline]
    fn apply_fill(&self, arr: &mut [f64]) {
        arr.iter_mut()
            .for_each(|density| *density = self.apply_density(*density));
    }

    pub fn create_new_ref(
        arg: ComponentReferenceImplementation<E>,
        data: &LinearUntypedData,
    ) -> ComponentReferenceImplementation<E> {
        match arg {
            ComponentReferenceImplementation::Shared(shared) => {
                ComponentReferenceImplementation::Shared(
                    LinearFunction::<NoEnvironment, SharedComponentReference>::new(
                        data.action,
                        shared,
                        data.arg,
                    )
                    .into(),
                )
            }
            ComponentReferenceImplementation::Mutable(owned) => {
                ComponentReferenceImplementation::Mutable(
                    LinearFunction::new(data.action, owned, data.arg).into(),
                )
            }
        }
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> ComponentFunctionImpl
    for LinearFunction<E, R>
{
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> MutableComponentFunctionImpl<E>
    for LinearFunction<E, R>
{
    #[inline]
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f64 {
        let density = self.input.sample_mut(pos, env);
        self.apply_density(density)
    }

    #[inline]
    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>) {
        self.input.fill_mut(arr, applier);
        self.apply_fill(arr);
    }

    fn environment(&self) -> ConverterEnvironment<E> {
        ConverterEnvironment::Linear(&self.input, &self.data)
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<E> {
        OwnedConverterEnvironment::Linear(self.input.wrapped_ref(), self.data)
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
    for LinearFunction<E, SharedComponentReference>
{
    #[inline]
    fn sample(&self, pos: &NoisePos) -> f64 {
        let density = self.input.sample(pos);
        self.apply_density(density)
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        self.input.fill(arr, applier);
        self.apply_fill(arr);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::Linear(&self.input, &self.data)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BinaryType {
    Mul,
    Add,
    Min,
    Max,
}

pub struct BinaryFunction<
    E: DensityFunctionEnvironment,
    R1: ComponentReference<E>,
    R2: ComponentReference<E>,
> {
    pub(crate) binary_type: BinaryType,
    pub(crate) arg1: R1,
    pub(crate) arg2: R2,
    _dummy: PhantomData<E>,
}

impl<E: DensityFunctionEnvironment, R1: ComponentReference<E>, R2: ComponentReference<E>>
    From<BinaryFunction<E, R1, R2>> for MutableComponentReference<E>
{
    fn from(value: BinaryFunction<E, R1, R2>) -> Self {
        Self(Box::new(value))
    }
}

impl<E: DensityFunctionEnvironment, R1: ComponentReference<E>, R2: ComponentReference<E>>
    BinaryFunction<E, R1, R2>
{
    pub fn new(binary_type: BinaryType, arg1: R1, arg2: R2) -> Self {
        Self {
            binary_type,
            arg1,
            arg2,
            _dummy: PhantomData::<E> {},
        }
    }

    #[inline]
    fn apply_densities(&self, density1: f64, density2: f64) -> f64 {
        match self.binary_type {
            BinaryType::Add => density1 + density2,
            BinaryType::Mul => density1 * density2,
            BinaryType::Min => density1.min(density2),
            BinaryType::Max => density1.max(density2),
        }
    }

    pub fn create_new_ref(
        arg1: ComponentReferenceImplementation<E>,
        arg2: ComponentReferenceImplementation<E>,
        action: BinaryType,
    ) -> ComponentReferenceImplementation<E> {
        match (arg1, arg2) {
            (
                ComponentReferenceImplementation::Shared(shared1),
                ComponentReferenceImplementation::Shared(shared2),
            ) => BinaryFunction::<E, SharedComponentReference, SharedComponentReference>::new(
                action, shared1, shared2,
            )
            .into(),
            (ref1, ref2) => {
                match_ref_implementations!(
                    (unwrapped1, ref1),
                    (unwrapped2, ref2);
                    {
                        ComponentReferenceImplementation::Mutable(
                            MutableComponentReference(Box::new(
                                BinaryFunction::new(action, unwrapped1, unwrapped2)
                            ))
                        )
                    }
                )
            }
        }
    }
}

impl<E: DensityFunctionEnvironment, R1: ComponentReference<E>, R2: ComponentReference<E>>
    ComponentFunctionImpl for BinaryFunction<E, R1, R2>
{
}

impl<E: DensityFunctionEnvironment, R1: ComponentReference<E>, R2: ComponentReference<E>>
    MutableComponentFunctionImpl<E> for BinaryFunction<E, R1, R2>
{
    #[inline]
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f64 {
        let density1 = self.arg1.sample_mut(pos, env);
        let density2 = self.arg2.sample_mut(pos, env);
        self.apply_densities(density1, density2)
    }

    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>) {
        self.arg1.fill_mut(arr, applier);
        match self.binary_type {
            BinaryType::Add => {
                let mut densities2 = vec![0f64; arr.len()];
                self.arg2.fill_mut(&mut densities2, applier);
                arr.iter_mut()
                    .zip(densities2)
                    .for_each(|(returned, temp)| *returned += temp);
            }
            BinaryType::Mul => {
                arr.iter_mut().enumerate().for_each(|(i, val)| {
                    if *val != 0f64 {
                        *val *= self.arg2.sample_mut(&applier.at(i), applier.env());
                    }
                });
            }
            BinaryType::Min => {
                arr.iter_mut().enumerate().for_each(|(i, val)| {
                    *val = val.min(self.arg2.sample_mut(&applier.at(i), applier.env()));
                });
            }
            BinaryType::Max => {
                arr.iter_mut().enumerate().for_each(|(i, val)| {
                    *val = val.max(self.arg2.sample_mut(&applier.at(i), applier.env()));
                });
            }
        }
    }

    fn environment(&self) -> ConverterEnvironment<E> {
        ConverterEnvironment::Binary(&self.arg1, &self.arg2, self.binary_type)
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<E> {
        OwnedConverterEnvironment::Binary(
            self.arg1.wrapped_ref(),
            self.arg2.wrapped_ref(),
            self.binary_type,
        )
    }

    fn convert(
        self: Box<Self>,
        converter: &mut dyn ConverterImpl<E>,
    ) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(
            self.arg1.convert(converter),
            self.arg2.convert(converter),
            self.binary_type,
        )
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(
            self.arg1.clone_to_new_ref(),
            self.arg2.clone_to_new_ref(),
            self.binary_type,
        )
    }
}

impl<E: DensityFunctionEnvironment> ImmutableComponentFunctionImpl
    for BinaryFunction<E, SharedComponentReference, SharedComponentReference>
{
    #[inline]
    fn sample(&self, pos: &NoisePos) -> f64 {
        let density1 = self.arg1.sample(pos);
        let density2 = self.arg2.sample(pos);
        self.apply_densities(density1, density2)
    }

    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        self.arg1.fill(arr, applier);
        match self.binary_type {
            BinaryType::Add => {
                let mut densities2 = vec![0f64; arr.len()];
                self.arg2.fill(&mut densities2, applier);
                arr.iter_mut()
                    .zip(densities2)
                    .for_each(|(returned, temp)| *returned += temp);
            }
            BinaryType::Mul => {
                arr.iter_mut().enumerate().for_each(|(i, val)| {
                    if *val != 0f64 {
                        *val *= self.arg2.sample(&applier.at(i));
                    }
                });
            }
            BinaryType::Min => {
                arr.iter_mut().enumerate().for_each(|(i, val)| {
                    *val = val.min(self.arg2.sample(&applier.at(i)));
                });
            }
            BinaryType::Max => {
                arr.iter_mut().enumerate().for_each(|(i, val)| {
                    *val = val.max(self.arg2.sample(&applier.at(i)));
                });
            }
        }
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::Binary(&self.arg1, &self.arg2, self.binary_type)
    }
}
