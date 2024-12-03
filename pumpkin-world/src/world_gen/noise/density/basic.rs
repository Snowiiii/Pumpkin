use std::{hash::Hash, marker::PhantomData};

use crate::{match_ref_implementations, world_gen::noise::clamped_map};

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

/// A density function that always returns the same value
pub struct ConstantFunction {
    value: f64,
}

impl ConstantFunction {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl ComponentFunctionImpl for ConstantFunction {}

impl ImmutableComponentFunctionImpl for ConstantFunction {
    #[inline]
    fn sample(&self, _pos: &NoisePos) -> f64 {
        self.value
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], _applier: &mut dyn ApplierImpl) {
        arr.fill(self.value);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::Constant(self.value)
    }
}

/// A density function that wraps another density function.
/// Primarily used to mark functions that should be modified by a converter
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum WrapperType {
    Cache2D,
    FlatCache,
    OnceCache,
    Interpolated,
    CellCache,
}

pub struct WrapperFunction<E: DensityFunctionEnvironment, R: ComponentReference<E>> {
    pub(crate) wrapped: R,
    pub(crate) wrapper_type: WrapperType,
    _dummy: PhantomData<E>,
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> From<WrapperFunction<E, R>>
    for MutableComponentReference<E>
{
    fn from(value: WrapperFunction<E, R>) -> Self {
        Self(Box::new(value))
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> WrapperFunction<E, R> {
    pub fn new(wrapped: R, wrapper_type: WrapperType) -> Self {
        Self {
            wrapped,
            wrapper_type,
            _dummy: PhantomData::<E> {},
        }
    }

    pub fn create_new_ref(
        wrapped: ComponentReferenceImplementation<E>,
        wrapper_type: WrapperType,
    ) -> ComponentReferenceImplementation<E> {
        match wrapped {
            ComponentReferenceImplementation::Shared(shared) => {
                ComponentReferenceImplementation::Shared(
                    WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                        shared,
                        wrapper_type,
                    )
                    .into(),
                )
            }
            ComponentReferenceImplementation::Mutable(owned) => {
                ComponentReferenceImplementation::Mutable(
                    WrapperFunction::new(owned, wrapper_type).into(),
                )
            }
        }
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> ComponentFunctionImpl
    for WrapperFunction<E, R>
{
}

impl<E: DensityFunctionEnvironment> ImmutableComponentFunctionImpl
    for WrapperFunction<E, SharedComponentReference>
{
    #[inline]
    fn sample(&self, pos: &NoisePos) -> f64 {
        self.wrapped.sample(pos)
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        self.wrapped.fill(arr, applier);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::Wrapper(&self.wrapped, self.wrapper_type)
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> MutableComponentFunctionImpl<E>
    for WrapperFunction<E, R>
{
    #[inline]
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f64 {
        self.wrapped.sample_mut(pos, env)
    }

    #[inline]
    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>) {
        self.wrapped.fill_mut(arr, applier);
    }

    fn environment(&self) -> ConverterEnvironment<E> {
        ConverterEnvironment::Wrapper(&self.wrapped, self.wrapper_type)
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<E> {
        OwnedConverterEnvironment::Wrapper(self.wrapped.wrapped_ref(), self.wrapper_type)
    }

    fn convert(
        self: Box<Self>,
        converter: &mut dyn ConverterImpl<E>,
    ) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(self.wrapped.convert(converter), self.wrapper_type)
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(self.wrapped.clone_to_new_ref(), self.wrapper_type)
    }
}

pub struct YClampedFunction {
    from: i32,
    to: i32,
    from_val: f64,
    to_val: f64,
}

impl YClampedFunction {
    pub fn new(from: i32, to: i32, from_val: f64, to_val: f64) -> Self {
        Self {
            from,
            to,
            from_val,
            to_val,
        }
    }
}

impl ComponentFunctionImpl for YClampedFunction {}

impl ImmutableComponentFunctionImpl for YClampedFunction {
    #[inline]
    fn sample(&self, pos: &NoisePos) -> f64 {
        clamped_map(
            pos.y() as f64,
            self.from as f64,
            self.to as f64,
            self.from_val,
            self.to_val,
        )
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        applier.fill(arr, self);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::YClamped(self)
    }
}

// TODO: Implement structures (this is a place holder for that)
pub struct BeardifierFunction {}

impl BeardifierFunction {
    pub const INSTANCE: Self = Self {};
}

impl ComponentFunctionImpl for BeardifierFunction {}

impl ImmutableComponentFunctionImpl for BeardifierFunction {
    fn sample(&self, _pos: &NoisePos) -> f64 {
        0f64
    }

    fn fill(&self, arr: &mut [f64], _applier: &mut dyn ApplierImpl) {
        arr.fill(0f64);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::Beardifier
    }
}

#[derive(Clone)]
pub(crate) struct RangeUntypedData {
    pub(crate) min: f64,
    pub(crate) max: f64,
}

pub struct RangeFunction<
    E: DensityFunctionEnvironment,
    R1: ComponentReference<E>,
    R2: ComponentReference<E>,
    R3: ComponentReference<E>,
> {
    pub(crate) input: R1,
    pub(crate) in_range: R2,
    pub(crate) out_range: R3,
    pub(crate) data: RangeUntypedData,
    _dummy: PhantomData<E>,
}

impl<
        E: DensityFunctionEnvironment,
        R1: ComponentReference<E>,
        R2: ComponentReference<E>,
        R3: ComponentReference<E>,
    > RangeFunction<E, R1, R2, R3>
{
    pub fn new(input: R1, min: f64, max: f64, in_range: R2, out_range: R3) -> Self {
        Self {
            input,
            in_range,
            out_range,
            data: RangeUntypedData { min, max },
            _dummy: PhantomData::<E> {},
        }
    }

    pub fn create_new_ref(
        input: ComponentReferenceImplementation<E>,
        in_sampler: ComponentReferenceImplementation<E>,
        out_sampler: ComponentReferenceImplementation<E>,
        data: &RangeUntypedData,
    ) -> ComponentReferenceImplementation<E> {
        match (input, in_sampler, out_sampler) {
            (
                ComponentReferenceImplementation::Shared(shared_input),
                ComponentReferenceImplementation::Shared(shared_in),
                ComponentReferenceImplementation::Shared(shared_out),
            ) => RangeFunction::<
                E,
                SharedComponentReference,
                SharedComponentReference,
                SharedComponentReference,
            >::new(shared_input, data.min, data.max, shared_in, shared_out)
            .into(),
            (input, in_sampler, out_sampler) => {
                match_ref_implementations!(
                    (input_ref, input),
                    (in_ref, in_sampler),
                    (out_ref, out_sampler);
                    {
                        ComponentReferenceImplementation::Mutable(
                            MutableComponentReference(Box::new(
                                RangeFunction::new(
                                    input_ref,
                                    data.min,
                                    data.max,
                                    in_ref,
                                    out_ref
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
    > From<RangeFunction<E, R1, R2, R3>> for MutableComponentReference<E>
{
    fn from(value: RangeFunction<E, R1, R2, R3>) -> Self {
        Self(Box::new(value))
    }
}

impl<
        E: DensityFunctionEnvironment,
        R1: ComponentReference<E>,
        R2: ComponentReference<E>,
        R3: ComponentReference<E>,
    > ComponentFunctionImpl for RangeFunction<E, R1, R2, R3>
{
}

impl<
        E: DensityFunctionEnvironment,
        R1: ComponentReference<E>,
        R2: ComponentReference<E>,
        R3: ComponentReference<E>,
    > MutableComponentFunctionImpl<E> for RangeFunction<E, R1, R2, R3>
{
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f64 {
        let sampled_density = self.input.sample_mut(pos, env);
        if sampled_density >= self.data.min && sampled_density < self.data.max {
            self.in_range.sample_mut(pos, env)
        } else {
            self.out_range.sample_mut(pos, env)
        }
    }

    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>) {
        self.input.fill_mut(arr, applier);
        arr.iter_mut().enumerate().for_each(|(i, val)| {
            if *val >= self.data.min && *val < self.data.max {
                *val = self.in_range.sample_mut(&applier.at(i), applier.env());
            } else {
                *val = self.out_range.sample_mut(&applier.at(i), applier.env());
            }
        });
    }

    fn environment(&self) -> ConverterEnvironment<E> {
        ConverterEnvironment::Range(&self.input, &self.in_range, &self.out_range, &self.data)
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<E> {
        OwnedConverterEnvironment::Range(
            self.input.wrapped_ref(),
            self.in_range.wrapped_ref(),
            self.out_range.wrapped_ref(),
            self.data,
        )
    }

    fn convert(
        self: Box<Self>,
        converter: &mut dyn ConverterImpl<E>,
    ) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(
            self.input.convert(converter),
            self.in_range.convert(converter),
            self.out_range.convert(converter),
            &self.data,
        )
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(
            self.input.clone_to_new_ref(),
            self.in_range.clone_to_new_ref(),
            self.out_range.clone_to_new_ref(),
            &self.data,
        )
    }
}

impl<E: DensityFunctionEnvironment> ImmutableComponentFunctionImpl
    for RangeFunction<
        E,
        SharedComponentReference,
        SharedComponentReference,
        SharedComponentReference,
    >
{
    fn sample(&self, pos: &NoisePos) -> f64 {
        let sampled_density = self.input.sample(pos);
        if sampled_density >= self.data.min && sampled_density < self.data.max {
            self.in_range.sample(pos)
        } else {
            self.out_range.sample(pos)
        }
    }

    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        self.input.fill(arr, applier);
        arr.iter_mut().enumerate().for_each(|(i, val)| {
            if *val >= self.data.min && *val < self.data.max {
                *val = self.in_range.sample(&applier.at(i));
            } else {
                *val = self.out_range.sample(&applier.at(i));
            }
        });
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::Range(&self.input, &self.in_range, &self.out_range, &self.data)
    }
}

#[cfg(test)]
mod test {
    use std::{fs, path::Path};

    use crate::{
        read_data_from_file,
        world_gen::noise::density::{
            component_functions::ImmutableComponentFunctionImpl, NoisePos, UnblendedNoisePos,
        },
    };

    use super::YClampedFunction;

    #[test]
    fn test_y_clamped() {
        let expected_data: Vec<f64> = read_data_from_file!("../../../../assets/y_clamp.json");
        let mut expected_iter = expected_data.iter();

        let function = YClampedFunction::new(-64, 320, 1.5, -1.5);
        for y in -64..=320 {
            let pos = NoisePos::Unblended(UnblendedNoisePos::new(0, y, 0));
            assert_eq!(function.sample(&pos), *expected_iter.next().unwrap());
        }
    }
}
