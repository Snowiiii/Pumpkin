use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

use super::{
    basic::{RangeFunction, RangeUntypedData, WrapperFunction, WrapperType, YClampedFunction},
    blend::BlendDensityFunction,
    math::{BinaryFunction, BinaryType, LinearFunction, LinearType, LinearUntypedData},
    noise::{
        InternalNoise, InterpolatedNoiseFunction, NoiseFunction, ShiftedNoiseFunction,
        ShiftedNoiseUntypedData,
    },
    offset::{ShiftAFunction, ShiftBFunction},
    spline::{ImmutableSplineRef, SplineFunction, SplineRef, SplineRefImpl},
    unary::{ClampFunction, ClampUntypedData, UnaryFunction, UnaryType},
    weird::{RarityMapper, WierdScaledFunction},
    NoisePos,
};

/// An environment for a mutable density function to reference. Supplied by the `EnvironmentApplierImpl`
pub trait DensityFunctionEnvironment: Send + Sync + 'static {}

/// A placeholder struct to mark a density function as having no environment.
pub struct NoEnvironment {}
impl DensityFunctionEnvironment for NoEnvironment {}

pub(crate) enum MutableComponentWrapper<E: DensityFunctionEnvironment> {
    Wrapper(ComponentReferenceImplementation<E>, WrapperType),
    BlendDensity(ComponentReferenceImplementation<E>),
    Linear(ComponentReferenceImplementation<E>),
    Binary(
        ComponentReferenceImplementation<E>,
        ComponentReferenceImplementation<E>,
    ),
    ShiftedNoise(
        ComponentReferenceImplementation<E>,
        ComponentReferenceImplementation<E>,
        ComponentReferenceImplementation<E>,
    ),
    Spline(SplineRef<E>),
    Clamp(ComponentReferenceImplementation<E>),
    Unary(ComponentReferenceImplementation<E>),
    Range(
        ComponentReferenceImplementation<E>,
        ComponentReferenceImplementation<E>,
        ComponentReferenceImplementation<E>,
    ),
    Wierd(ComponentReferenceImplementation<E>),
}

#[macro_export]
macro_rules! match_ref_implementations {
    (($name:ident, $value:expr); $builder:expr) => {
        match $value {
            ComponentReferenceImplementation::Shared($name) => {$builder},
            ComponentReferenceImplementation::Mutable($name) => {$builder},
        }
    };
    (($name:ident, $value:expr), $(($name2:ident, $value2:expr)),+; $builder:expr) => {
        match $value {
            ComponentReferenceImplementation::Shared($name) => {
                match_ref_implementations!($(($name2, $value2)),+;$builder)
            }
            ComponentReferenceImplementation::Mutable($name) => {
                match_ref_implementations!($(($name2, $value2)),+;$builder)
            }
        }
    };
}

pub(crate) enum SharedConverterEnvironment<'a> {
    Constant(f64),
    Wrapper(&'a SharedComponentReference, WrapperType),
    YClamped(&'a YClampedFunction),
    Beardifier,
    Range(
        &'a SharedComponentReference,
        &'a SharedComponentReference,
        &'a SharedComponentReference,
        &'a RangeUntypedData,
    ),
    BlendOffset,
    BlendAlpha,
    BlendDensity(&'a SharedComponentReference),
    End,
    Linear(&'a SharedComponentReference, &'a LinearUntypedData),
    Binary(
        &'a SharedComponentReference,
        &'a SharedComponentReference,
        BinaryType,
    ),
    Noise(&'a NoiseFunction),
    ShiftedNoise(
        &'a SharedComponentReference,
        &'a SharedComponentReference,
        &'a SharedComponentReference,
        &'a Arc<InternalNoise>,
        &'a ShiftedNoiseUntypedData,
    ),
    InterpolatedNoise(&'a InterpolatedNoiseFunction),
    ShiftA(&'a Arc<InternalNoise>),
    ShiftB(&'a Arc<InternalNoise>),
    Spline(&'a ImmutableSplineRef),
    Clamp(&'a SharedComponentReference, &'a ClampUntypedData),
    Unary(&'a SharedComponentReference, UnaryType),
    Wierd(
        &'a SharedComponentReference,
        &'a Arc<InternalNoise>,
        RarityMapper,
    ),
}

impl<'a> SharedConverterEnvironment<'a> {
    fn as_env<E: DensityFunctionEnvironment>(&self) -> ConverterEnvironment<'a, E> {
        match self {
            Self::Constant(val) => ConverterEnvironment::Constant(*val),
            Self::Wrapper(reference, action) => ConverterEnvironment::Wrapper(*reference, *action),
            Self::YClamped(func) => ConverterEnvironment::YClamped(func),
            Self::Beardifier => ConverterEnvironment::Beardifier,
            Self::Range(f1, f2, f3, data) => ConverterEnvironment::Range(*f1, *f2, *f3, data),
            Self::BlendOffset => ConverterEnvironment::BlendOffset,
            Self::BlendAlpha => ConverterEnvironment::BlendAlpha,
            Self::BlendDensity(f) => ConverterEnvironment::BlendDensity(*f),
            Self::End => ConverterEnvironment::End,
            Self::Linear(f, data) => ConverterEnvironment::Linear(*f, data),
            Self::Binary(f1, f2, data) => ConverterEnvironment::Binary(*f1, *f2, *data),
            Self::Noise(n) => ConverterEnvironment::Noise(n),
            Self::ShiftedNoise(x, y, z, noise, data) => {
                ConverterEnvironment::ShiftedNoise(*x, *y, *z, noise, data)
            }
            Self::InterpolatedNoise(func) => ConverterEnvironment::InterpolatedNoise(func),
            Self::ShiftA(noise) => ConverterEnvironment::ShiftA(noise),
            Self::ShiftB(noise) => ConverterEnvironment::ShiftB(noise),
            Self::Spline(spline) => ConverterEnvironment::Spline(*spline),
            Self::Clamp(f, data) => ConverterEnvironment::Clamp(*f, data),
            Self::Unary(f, action) => ConverterEnvironment::Unary(*f, *action),
            Self::Wierd(f, noise, rarity) => ConverterEnvironment::Wierd(*f, noise, *rarity),
        }
    }

    fn maybe_into_env<E: DensityFunctionEnvironment>(self) -> Option<OwnedConverterEnvironment<E>> {
        Some(match self {
            Self::Wrapper(reference, action) => {
                OwnedConverterEnvironment::Wrapper(reference.clone_to_new_ref(), action)
            }
            Self::Range(f1, f2, f3, data) => OwnedConverterEnvironment::Range(
                f1.clone_to_new_ref(),
                f2.clone_to_new_ref(),
                f3.clone_to_new_ref(),
                data.clone(),
            ),
            Self::BlendDensity(f) => OwnedConverterEnvironment::BlendDensity(f.clone_to_new_ref()),
            Self::Linear(f, data) => {
                OwnedConverterEnvironment::Linear(f.clone_to_new_ref(), data.clone())
            }
            Self::Binary(f1, f2, data) => OwnedConverterEnvironment::Binary(
                f1.clone_to_new_ref(),
                f2.clone_to_new_ref(),
                data,
            ),
            Self::ShiftedNoise(x, y, z, noise, data) => OwnedConverterEnvironment::ShiftedNoise(
                x.clone_to_new_ref(),
                y.clone_to_new_ref(),
                z.clone_to_new_ref(),
                noise.clone(),
                data.clone(),
            ),
            Self::Spline(spline) => {
                OwnedConverterEnvironment::Spline(SplineRef::Immutable(spline.clone()))
            }
            Self::Clamp(f, data) => {
                OwnedConverterEnvironment::Clamp(f.clone_to_new_ref(), data.clone())
            }
            Self::Unary(f, action) => {
                OwnedConverterEnvironment::Unary(f.clone_to_new_ref(), action)
            }
            Self::Wierd(f, noise, rarity) => {
                OwnedConverterEnvironment::Wierd(f.clone_to_new_ref(), noise.clone(), rarity)
            }
            _ => {
                return None;
            }
        })
    }

    fn internal_conversion<E: DensityFunctionEnvironment>(
        &self,
        converter: &mut dyn ConverterImpl<E>,
    ) -> Option<ComponentReferenceImplementation<E>> {
        match self {
            Self::Wrapper(reference, action) => reference.maybe_convert(converter).map(|new_ref| {
                WrapperFunction::<E, SharedComponentReference>::create_new_ref(new_ref, *action)
            }),
            Self::Range(input, in_range, out_range, data) => {
                let conv_input = input.maybe_convert(converter);
                let conv_in = in_range.maybe_convert(converter);
                let conv_out = out_range.maybe_convert(converter);
                match (conv_input, conv_in, conv_out) {
                    (None, None, None) => None,
                    (f1, f2, f3) => {
                        let input = f1.unwrap_or_else(|| (*input).clone().into());
                        let in_sampler = f2.unwrap_or_else(|| (*in_range).clone().into());
                        let out_sampler = f3.unwrap_or_else(|| (*out_range).clone().into());
                        Some(RangeFunction::<
                            E,
                            SharedComponentReference,
                            SharedComponentReference,
                            SharedComponentReference,
                        >::create_new_ref(
                            input, in_sampler, out_sampler, data
                        ))
                    }
                }
            }
            Self::BlendDensity(reference) => reference.maybe_convert(converter).map(|new_ref| {
                BlendDensityFunction::<E, SharedComponentReference>::create_new_ref(new_ref)
            }),
            Self::Linear(reference, data) => reference.maybe_convert(converter).map(|new_ref| {
                LinearFunction::<E, SharedComponentReference>::create_new_ref(new_ref, data)
            }),
            Self::Binary(arg1, arg2, action) => {
                let conv_arg1 = arg1.maybe_convert(converter);
                let conv_arg2 = arg2.maybe_convert(converter);
                match (conv_arg1, conv_arg2) {
                    (None, None) => None,
                    (f1, f2) => {
                        let arg1 = f1.unwrap_or_else(|| (*arg1).clone().into());
                        let arg2 = f2.unwrap_or_else(|| (*arg2).clone().into());

                        Some(BinaryFunction::<
                            E,
                            SharedComponentReference,
                            SharedComponentReference,
                        >::create_new_ref(
                            arg1, arg2, *action
                        ))
                    }
                }
            }
            Self::Noise(noise) => converter.convert_noise(&noise.noise).map(|new_noise| {
                NoiseFunction::new(new_noise, noise.xz_scale, noise.y_scale).into()
            }),
            Self::ShiftedNoise(x, y, z, noise, data) => {
                let conv_x = x.maybe_convert(converter);
                let conv_y = y.maybe_convert(converter);
                let conv_z = z.maybe_convert(converter);
                let conv_noise = converter.convert_noise(noise);

                match (conv_x, conv_y, conv_z, conv_noise) {
                    (None, None, None, None) => None,
                    (f1, f2, f3, maybe_noise) => {
                        let x = f1.unwrap_or_else(|| (*x).clone().into());
                        let y = f2.unwrap_or_else(|| (*y).clone().into());
                        let z = f3.unwrap_or_else(|| (*z).clone().into());
                        let noise = maybe_noise.unwrap_or_else(|| (*noise).clone());

                        Some(ShiftedNoiseFunction::<
                            E,
                            SharedComponentReference,
                            SharedComponentReference,
                            SharedComponentReference,
                        >::create_new_ref(
                            x, y, z, noise, data
                        ))
                    }
                }
            }
            Self::ShiftA(noise) => converter
                .convert_noise(noise)
                .map(|new_noise| ShiftAFunction::new(new_noise).into()),
            Self::ShiftB(noise) => converter
                .convert_noise(noise)
                .map(|new_noise| ShiftBFunction::new(new_noise).into()),
            Self::Spline(spline) => spline.maybe_convert(converter).map(|new_spline| {
                SplineFunction::<E, ImmutableSplineRef>::create_new_ref(new_spline)
            }),
            Self::Clamp(f, data) => f.maybe_convert(converter).map(|new_ref| {
                ClampFunction::<E, SharedComponentReference>::create_new_ref(new_ref, data)
            }),
            Self::Unary(f, action) => f.maybe_convert(converter).map(|new_ref| {
                UnaryFunction::<E, SharedComponentReference>::create_new_ref(new_ref, *action)
            }),
            Self::Wierd(f, noise, rarity) => {
                let conv_f = f.maybe_convert(converter);
                let conv_noise = converter.convert_noise(noise);
                match (conv_f, conv_noise) {
                    (None, None) => None,
                    (maybe_f, maybe_noise) => {
                        let f = maybe_f.unwrap_or_else(|| (*f).clone().into());
                        let noise = maybe_noise.unwrap_or_else(|| (*noise).clone());
                        Some(
                            WierdScaledFunction::<E, SharedComponentReference>::create_new_ref(
                                f, noise, *rarity,
                            ),
                        )
                    }
                }
            }
            _ => None,
        }
    }
}

pub(crate) enum ConverterEnvironment<'a, E: DensityFunctionEnvironment> {
    Constant(f64),
    Wrapper(&'a dyn ComponentReference<E>, WrapperType),
    YClamped(&'a YClampedFunction),
    Beardifier,
    Range(
        &'a dyn ComponentReference<E>,
        &'a dyn ComponentReference<E>,
        &'a dyn ComponentReference<E>,
        &'a RangeUntypedData,
    ),
    BlendOffset,
    BlendAlpha,
    BlendDensity(&'a dyn ComponentReference<E>),
    End,
    Linear(&'a dyn ComponentReference<E>, &'a LinearUntypedData),
    Binary(
        &'a dyn ComponentReference<E>,
        &'a dyn ComponentReference<E>,
        BinaryType,
    ),
    Noise(&'a NoiseFunction),
    ShiftedNoise(
        &'a dyn ComponentReference<E>,
        &'a dyn ComponentReference<E>,
        &'a dyn ComponentReference<E>,
        &'a Arc<InternalNoise>,
        &'a ShiftedNoiseUntypedData,
    ),
    InterpolatedNoise(&'a InterpolatedNoiseFunction),
    ShiftA(&'a Arc<InternalNoise>),
    ShiftB(&'a Arc<InternalNoise>),
    Spline(&'a dyn SplineRefImpl<E>),
    Clamp(&'a dyn ComponentReference<E>, &'a ClampUntypedData),
    Unary(&'a dyn ComponentReference<E>, UnaryType),
    Wierd(
        &'a dyn ComponentReference<E>,
        &'a Arc<InternalNoise>,
        RarityMapper,
    ),
    ChunkNoise,
}

pub(crate) enum ConversionResultPre<E: DensityFunctionEnvironment> {
    New(ComponentReferenceImplementation<E>),
    NoChange,
    Default,
}

/// Only for components that reference other components
pub(crate) enum OwnedConverterEnvironment<E: DensityFunctionEnvironment> {
    Wrapper(ComponentReferenceImplementation<E>, WrapperType),
    Range(
        ComponentReferenceImplementation<E>,
        ComponentReferenceImplementation<E>,
        ComponentReferenceImplementation<E>,
        RangeUntypedData,
    ),
    BlendDensity(ComponentReferenceImplementation<E>),
    Linear(ComponentReferenceImplementation<E>, LinearUntypedData),
    Binary(
        ComponentReferenceImplementation<E>,
        ComponentReferenceImplementation<E>,
        BinaryType,
    ),
    ShiftedNoise(
        ComponentReferenceImplementation<E>,
        ComponentReferenceImplementation<E>,
        ComponentReferenceImplementation<E>,
        Arc<InternalNoise>,
        ShiftedNoiseUntypedData,
    ),
    Spline(SplineRef<E>),
    Clamp(ComponentReferenceImplementation<E>, ClampUntypedData),
    Unary(ComponentReferenceImplementation<E>, UnaryType),
    Wierd(
        ComponentReferenceImplementation<E>,
        Arc<InternalNoise>,
        RarityMapper,
    ),
}

impl<E: DensityFunctionEnvironment> OwnedConverterEnvironment<E> {
    pub fn rebuild_reference(self) -> ComponentReferenceImplementation<E> {
        match self {
            Self::Wrapper(f, t) => {
                WrapperFunction::<E, SharedComponentReference>::create_new_ref(f, t)
            }
            Self::Wierd(f, n, r) => {
                WierdScaledFunction::<E, SharedComponentReference>::create_new_ref(f, n, r)
            }
            Self::Unary(f, t) => UnaryFunction::<E, SharedComponentReference>::create_new_ref(f, t),
            Self::Clamp(f, t) => {
                ClampFunction::<E, SharedComponentReference>::create_new_ref(f, &t)
            }
            Self::Spline(s) => SplineFunction::<E, ImmutableSplineRef>::create_new_ref(s),
            Self::ShiftedNoise(x, y, z, n, d) => ShiftedNoiseFunction::<
                E,
                SharedComponentReference,
                SharedComponentReference,
                SharedComponentReference,
            >::create_new_ref(x, y, z, n, &d),
            Self::Binary(x, y, t) => BinaryFunction::<
                E,
                SharedComponentReference,
                SharedComponentReference,
            >::create_new_ref(x, y, t),
            Self::Linear(x, t) => {
                LinearFunction::<E, SharedComponentReference>::create_new_ref(x, &t)
            }
            Self::BlendDensity(f) => {
                BlendDensityFunction::<E, SharedComponentReference>::create_new_ref(f)
            }
            Self::Range(x, y, z, d) => RangeFunction::<
                E,
                SharedComponentReference,
                SharedComponentReference,
                SharedComponentReference,
            >::create_new_ref(x, y, z, &d),
        }
    }
}

// TODO: Also just generically make this better
pub trait ConverterImpl<E: DensityFunctionEnvironment> {
    /// Takes action before internal density functions are converted
    fn convert_env_pre_internal(
        &mut self,
        component: ConverterEnvironment<E>,
    ) -> ConversionResultPre<E>;

    fn converts_post_internal(&mut self, component: ConverterEnvironment<E>) -> bool;

    /// Takes action after internal density functions are converted
    fn convert_env_post_internal(
        &mut self,
        component: OwnedConverterEnvironment<E>,
    ) -> ComponentReferenceImplementation<E>;

    fn convert_noise(&mut self, noise: &Arc<InternalNoise>) -> Option<Arc<InternalNoise>>;
}

/// Fills the `arr` given a mutable density function and environment
pub trait EnvironmentApplierImpl: ApplierImpl {
    type Env: DensityFunctionEnvironment;

    fn env(&mut self) -> &Self::Env;

    fn fill_mut(
        &mut self,
        arr: &mut [f64],
        function: &mut dyn MutableComponentFunctionImpl<Self::Env>,
    );

    fn cast_up(&mut self) -> &mut dyn ApplierImpl;
}

/// Fills the `arr` given a immutable density function
pub trait ApplierImpl {
    fn at(&mut self, index: usize) -> NoisePos;

    fn fill(&mut self, arr: &mut [f64], function: &dyn ImmutableComponentFunctionImpl);
}

/// Methods shared across immutable and mutable density functions
pub trait ComponentFunctionImpl: Send + Sync {}

/// A density function that needs no environment or mutable state. Only has `ComponentReference`s.
pub trait ImmutableComponentFunctionImpl: ComponentFunctionImpl {
    fn sample(&self, pos: &NoisePos) -> f64;

    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl);

    fn shared_environment(&self) -> SharedConverterEnvironment;
}

/// A density function that needs a mutable state (and possibly an environment). May have
/// `ComponentReference`s or `MutableComponentReference`s.
pub trait MutableComponentFunctionImpl<E: DensityFunctionEnvironment>:
    ComponentFunctionImpl
{
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f64;

    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>);

    fn environment(&self) -> ConverterEnvironment<E>;

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<E>;

    fn convert(
        self: Box<Self>,
        converter: &mut dyn ConverterImpl<E>,
    ) -> ComponentReferenceImplementation<E>;

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E>;
}

/// Basic functions for simple modifications to a current component
pub trait ComponentReferenceMap<E: DensityFunctionEnvironment> {
    type Result: ComponentReference<E>;

    fn clamp(self, min: f64, max: f64) -> Self::Result;

    fn abs(self) -> Self::Result;

    fn square(self) -> Self::Result;

    fn cube(self) -> Self::Result;

    fn half_negative(self) -> Self::Result;

    fn quarter_negative(self) -> Self::Result;

    fn squeeze(self) -> Self::Result;

    fn add_const(self, other: f64) -> Self::Result;

    fn mul_const(self, other: f64) -> Self::Result;
}

pub trait ComponentReferenceMath<E: DensityFunctionEnvironment, R: ComponentReference<E>> {
    type Result: ComponentReference<E>;

    fn add(self, other: R) -> Self::Result;

    fn mul(self, other: R) -> Self::Result;

    fn min(self, other: R) -> Self::Result;

    fn max(self, other: R) -> Self::Result;
}

pub(crate) enum ComponentReferenceImplementation<E: DensityFunctionEnvironment> {
    Shared(SharedComponentReference),
    Mutable(MutableComponentReference<E>),
}

impl<E: DensityFunctionEnvironment> ComponentReferenceImplementation<E> {
    pub fn add(
        self,
        other: ComponentReferenceImplementation<E>,
    ) -> ComponentReferenceImplementation<E> {
        match (self, other) {
            (
                ComponentReferenceImplementation::Shared(shared1),
                ComponentReferenceImplementation::Shared(shared2),
            ) => BinaryFunction::<E, SharedComponentReference, SharedComponentReference>::new(
                BinaryType::Add,
                shared1,
                shared2,
            )
            .into(),
            (ref1, ref2) =>
            BinaryFunction::<E, SharedComponentReference, SharedComponentReference>::create_new_ref(ref1, ref2, BinaryType::Add),
        }
    }
}

impl<E: DensityFunctionEnvironment> ComponentReferenceImplementation<E> {
    #[inline]
    pub fn environment(&self) -> ConverterEnvironment<E> {
        match self {
            Self::Shared(shared) => shared.environment(),
            Self::Mutable(owned) => owned.environment(),
        }
    }

    #[inline]
    pub fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E> {
        match self {
            Self::Shared(shared) => shared.clone_to_new_ref(),
            Self::Mutable(owned) => owned.clone_to_new_ref(),
        }
    }

    #[inline]
    pub fn into_environment(
        self,
    ) -> Result<OwnedConverterEnvironment<E>, ComponentReferenceImplementation<E>> {
        match self {
            Self::Shared(shared) => shared.into_environment(),
            Self::Mutable(owned) => owned.into_environment(),
        }
    }

    #[inline]
    pub fn convert(self, converter: &mut dyn ConverterImpl<E>) -> Self {
        match self {
            Self::Shared(shared) => shared.convert(converter),
            Self::Mutable(owned) => owned.convert(converter),
        }
    }

    #[inline]
    pub fn assert_shared(self) -> SharedComponentReference {
        match self {
            Self::Shared(shared) => shared,
            Self::Mutable(_) => unreachable!(),
        }
    }

    #[inline]
    pub fn boxed(self) -> Box<dyn ComponentReference<E>> {
        match self {
            Self::Shared(shared) => Box::new(shared),
            Self::Mutable(owned) => Box::new(owned),
        }
    }
}

impl<E: DensityFunctionEnvironment> From<MutableComponentReference<E>>
    for ComponentReferenceImplementation<E>
{
    fn from(value: MutableComponentReference<E>) -> Self {
        Self::Mutable(value)
    }
}

impl<E: DensityFunctionEnvironment> From<SharedComponentReference>
    for ComponentReferenceImplementation<E>
{
    fn from(value: SharedComponentReference) -> Self {
        Self::Shared(value)
    }
}

impl<E: DensityFunctionEnvironment, F: ImmutableComponentFunctionImpl + 'static> From<F>
    for ComponentReferenceImplementation<E>
{
    fn from(value: F) -> Self {
        Self::Shared(value.into())
    }
}

/// A reference to some other density function whether it be immutable or mutable
pub trait ComponentReference<E: DensityFunctionEnvironment>: 'static + Send + Sync {
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f64;

    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>);

    fn convert(self, converter: &mut dyn ConverterImpl<E>) -> ComponentReferenceImplementation<E>;

    fn matches_ref(&self, reference: &ComponentReferenceImplementation<E>) -> bool;

    fn environment(&self) -> ConverterEnvironment<E>;

    /// Returns an environment if able to be converted, otherwise itself as a reference wrapper
    fn into_environment(
        self,
    ) -> Result<OwnedConverterEnvironment<E>, ComponentReferenceImplementation<E>>;

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E>;

    fn wrapped_ref(self) -> ComponentReferenceImplementation<E>;

    fn wrapped_ref_from_box(self: Box<Self>) -> ComponentReferenceImplementation<E>;
}

/// A shared reference to an immutable density function
#[derive(Clone)]
pub struct SharedComponentReference(pub(crate) Arc<dyn ImmutableComponentFunctionImpl>);

impl PartialEq for SharedComponentReference {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for SharedComponentReference {}

impl Hash for SharedComponentReference {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Arc::as_ptr(&self.0);
        let addr = ptr.cast::<()>() as usize;
        addr.hash(state);
    }
}

impl<F: ImmutableComponentFunctionImpl + 'static> From<F> for SharedComponentReference {
    fn from(value: F) -> Self {
        Self(Arc::new(value))
    }
}

impl SharedComponentReference {
    #[inline]
    pub fn sample(&self, pos: &NoisePos) -> f64 {
        self.0.sample(pos)
    }

    #[inline]
    pub fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        self.0.fill(arr, applier);
    }

    /// Returns None if no changes have been made
    pub(crate) fn maybe_convert<E: DensityFunctionEnvironment>(
        &self,
        converter: &mut dyn ConverterImpl<E>,
    ) -> Option<ComponentReferenceImplementation<E>> {
        let shared_env = self.0.shared_environment();
        match converter.convert_env_pre_internal(shared_env.as_env()) {
            ConversionResultPre::New(reference) => Some(reference),
            ConversionResultPre::NoChange => None,
            ConversionResultPre::Default => {
                let internal_conversion = shared_env
                    .internal_conversion(converter)
                    .unwrap_or_else(|| self.clone().into());

                Some(
                    if converter.converts_post_internal(internal_conversion.environment()) {
                        let env = internal_conversion
                            .into_environment()
                            .unwrap_or_else(|_| panic!());
                        converter.convert_env_post_internal(env)
                    } else {
                        internal_conversion
                    },
                )
            }
        }
    }
}

impl<E: DensityFunctionEnvironment> ComponentReference<E> for SharedComponentReference {
    #[inline]
    fn sample_mut(&mut self, pos: &NoisePos, _env: &E) -> f64 {
        self.sample(pos)
    }

    #[inline]
    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>) {
        self.fill(arr, applier.cast_up());
    }

    fn convert(self, converter: &mut dyn ConverterImpl<E>) -> ComponentReferenceImplementation<E> {
        self.maybe_convert(converter).unwrap_or_else(|| self.into())
    }

    fn environment(&self) -> ConverterEnvironment<E> {
        self.0.shared_environment().as_env()
    }

    fn into_environment(
        self,
    ) -> Result<OwnedConverterEnvironment<E>, ComponentReferenceImplementation<E>> {
        self.0
            .shared_environment()
            .maybe_into_env()
            .ok_or_else(|| self.into())
    }

    fn matches_ref(&self, reference: &ComponentReferenceImplementation<E>) -> bool {
        match reference {
            ComponentReferenceImplementation::Shared(shared) => Arc::ptr_eq(&self.0, &shared.0),
            ComponentReferenceImplementation::Mutable(_) => false,
        }
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E> {
        self.clone().into()
    }

    fn wrapped_ref(self) -> ComponentReferenceImplementation<E> {
        self.into()
    }

    fn wrapped_ref_from_box(self: Box<Self>) -> ComponentReferenceImplementation<E> {
        (*self).into()
    }
}

impl<I: Into<SharedComponentReference>> ComponentReferenceMap<NoEnvironment> for I {
    type Result = SharedComponentReference;

    fn abs(self) -> Self::Result {
        SharedComponentReference(Arc::new(UnaryFunction::<
            NoEnvironment,
            SharedComponentReference,
        >::new(UnaryType::Abs, self.into())))
    }

    fn cube(self) -> Self::Result {
        SharedComponentReference(Arc::new(UnaryFunction::<
            NoEnvironment,
            SharedComponentReference,
        >::new(UnaryType::Cube, self.into())))
    }

    fn clamp(self, min: f64, max: f64) -> Self::Result {
        #[cfg(debug_assertions)]
        assert!(min <= max);
        SharedComponentReference(Arc::new(ClampFunction::<
            NoEnvironment,
            SharedComponentReference,
        >::new(self.into(), min, max)))
    }

    fn square(self) -> Self::Result {
        SharedComponentReference(Arc::new(UnaryFunction::<
            NoEnvironment,
            SharedComponentReference,
        >::new(UnaryType::Square, self.into())))
    }

    fn squeeze(self) -> Self::Result {
        SharedComponentReference(Arc::new(UnaryFunction::<
            NoEnvironment,
            SharedComponentReference,
        >::new(UnaryType::Squeeze, self.into())))
    }

    fn half_negative(self) -> Self::Result {
        SharedComponentReference(Arc::new(UnaryFunction::<
            NoEnvironment,
            SharedComponentReference,
        >::new(UnaryType::HalfNeg, self.into())))
    }

    fn quarter_negative(self) -> Self::Result {
        SharedComponentReference(Arc::new(UnaryFunction::<
            NoEnvironment,
            SharedComponentReference,
        >::new(UnaryType::QuartNeg, self.into())))
    }

    fn add_const(self, other: f64) -> Self::Result {
        SharedComponentReference(Arc::new(LinearFunction::<
            NoEnvironment,
            SharedComponentReference,
        >::new(LinearType::Add, self.into(), other)))
    }

    fn mul_const(self, other: f64) -> Self::Result {
        SharedComponentReference(Arc::new(LinearFunction::<
            NoEnvironment,
            SharedComponentReference,
        >::new(LinearType::Mul, self.into(), other)))
    }
}

impl<I: Into<SharedComponentReference>>
    ComponentReferenceMath<NoEnvironment, SharedComponentReference> for I
{
    type Result = SharedComponentReference;

    fn add(self, other: SharedComponentReference) -> Self::Result {
        SharedComponentReference(Arc::new(BinaryFunction::<
            NoEnvironment,
            SharedComponentReference,
            SharedComponentReference,
        >::new(BinaryType::Add, self.into(), other)))
    }

    fn mul(self, other: SharedComponentReference) -> Self::Result {
        SharedComponentReference(Arc::new(BinaryFunction::<
            NoEnvironment,
            SharedComponentReference,
            SharedComponentReference,
        >::new(BinaryType::Mul, self.into(), other)))
    }

    fn min(self, other: SharedComponentReference) -> Self::Result {
        SharedComponentReference(Arc::new(BinaryFunction::<
            NoEnvironment,
            SharedComponentReference,
            SharedComponentReference,
        >::new(BinaryType::Min, self.into(), other)))
    }

    fn max(self, other: SharedComponentReference) -> Self::Result {
        SharedComponentReference(Arc::new(BinaryFunction::<
            NoEnvironment,
            SharedComponentReference,
            SharedComponentReference,
        >::new(BinaryType::Max, self.into(), other)))
    }
}
/// A owned reference to a mutable density function
pub struct MutableComponentReference<E: DensityFunctionEnvironment>(
    pub(crate) Box<dyn MutableComponentFunctionImpl<E>>,
);

impl<E: DensityFunctionEnvironment> ComponentReference<E> for MutableComponentReference<E> {
    #[inline]
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f64 {
        self.0.sample_mut(pos, env)
    }

    #[inline]
    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>) {
        self.0.fill_mut(arr, applier);
    }

    fn environment(&self) -> ConverterEnvironment<E> {
        self.0.environment()
    }

    fn into_environment(
        self,
    ) -> Result<OwnedConverterEnvironment<E>, ComponentReferenceImplementation<E>> {
        Ok(self.0.into_environment())
    }

    fn convert(self, converter: &mut dyn ConverterImpl<E>) -> ComponentReferenceImplementation<E> {
        let env = self.0.environment();
        match converter.convert_env_pre_internal(env) {
            ConversionResultPre::New(function) => function,
            ConversionResultPre::NoChange => self.into(),
            ConversionResultPre::Default => {
                let internal = self.0.convert(converter);

                if converter.converts_post_internal(internal.environment()) {
                    let env = internal.into_environment().unwrap_or_else(|_| panic!());
                    converter.convert_env_post_internal(env)
                } else {
                    internal
                }
            }
        }
    }

    fn matches_ref(&self, reference: &ComponentReferenceImplementation<E>) -> bool {
        match reference {
            ComponentReferenceImplementation::Shared(_) => false,
            ComponentReferenceImplementation::Mutable(_owned) => {
                //TODO: A way to compare owned components (do we even need this?)
                false
            }
        }
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E> {
        self.0.clone_to_new_ref()
    }

    fn wrapped_ref(self) -> ComponentReferenceImplementation<E> {
        ComponentReferenceImplementation::Mutable(self)
    }

    fn wrapped_ref_from_box(self: Box<Self>) -> ComponentReferenceImplementation<E> {
        ComponentReferenceImplementation::Mutable(*self)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use pumpkin_core::random::{legacy_rand::LegacyRand, RandomDeriver, RandomImpl};

    use crate::world_gen::noise::{
        built_in_noise_params,
        density::{
            noise::{InternalNoise, NoiseFunction},
            spline::{FloatAmplifier, ImmutableSplineRef, SplineBuilder, SplineFunction},
            test::{FakeEnvironment, OwnedConverter, TestConverter},
            NoisePos, UnblendedNoisePos,
        },
    };

    use super::{ComponentReference, NoEnvironment, SharedComponentReference};

    #[test]
    fn test_owned_minimal_conversion_spline() {
        let minimal_spline = SplineBuilder::new(
            NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    &built_in_noise_params::NETHER_WART,
                    None,
                )),
                0.2f64,
                1.1f64,
            )
            .into(),
            FloatAmplifier::Identity,
        )
        .add_fixed_value(0f32, 1f32)
        .build();

        let test_func: SharedComponentReference =
            SplineFunction::<NoEnvironment, ImmutableSplineRef>::new(minimal_spline.into()).into();

        let mut rand = LegacyRand::from_seed(0);
        let splitter = rand.next_splitter();
        let mut converter = TestConverter {
            splitter: RandomDeriver::Legacy(splitter),
        };
        let standard_convert = test_func.clone().convert(&mut converter).assert_shared();

        let mut rand = LegacyRand::from_seed(0);
        let splitter = rand.next_splitter();
        let mut converter = OwnedConverter {
            splitter: RandomDeriver::Legacy(splitter),
        };
        let mut owned_convert = test_func.convert_to_dyn(&mut converter);

        for i in -10..=10 {
            for j in -10..=10 {
                for k in -10..=10 {
                    let pos =
                        NoisePos::Unblended(UnblendedNoisePos::new(i * 100, j * 100, k * 100));
                    assert_eq!(
                        standard_convert.sample(&pos),
                        owned_convert.sample_mut(&pos, &FakeEnvironment {})
                    );
                }
            }
        }
    }

    #[test]
    fn test_nested_minimal_conversion_spline() {
        let minimal_spline = SplineBuilder::new(
            NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    &built_in_noise_params::NETHER_WART,
                    None,
                )),
                0.2f64,
                1.1f64,
            )
            .into(),
            FloatAmplifier::Identity,
        )
        .add_spline_value(
            0f32,
            SplineBuilder::new(
                NoiseFunction::new(
                    Arc::new(InternalNoise::new(
                        &built_in_noise_params::NETHER_WART,
                        None,
                    )),
                    0.2f64,
                    1.1f64,
                )
                .into(),
                FloatAmplifier::Identity,
            )
            .add_fixed_value(-1f32, -1f32)
            .add_fixed_value(1f32, 1f32)
            .build()
            .into(),
        )
        .add_fixed_value(1f32, 1f32)
        .build();

        let test_func: SharedComponentReference =
            SplineFunction::<NoEnvironment, ImmutableSplineRef>::new(minimal_spline.into()).into();

        let mut rand = LegacyRand::from_seed(0);
        let splitter = rand.next_splitter();
        let mut converter = TestConverter {
            splitter: RandomDeriver::Legacy(splitter),
        };
        let standard_convert = test_func.clone().convert(&mut converter).assert_shared();

        let mut rand = LegacyRand::from_seed(0);
        let splitter = rand.next_splitter();
        let mut converter = OwnedConverter {
            splitter: RandomDeriver::Legacy(splitter),
        };
        let mut owned_convert = test_func.convert_to_dyn(&mut converter);

        for i in -10..=10 {
            for j in -10..=10 {
                for k in -10..=10 {
                    let pos =
                        NoisePos::Unblended(UnblendedNoisePos::new(i * 100, j * 100, k * 100));
                    assert_eq!(
                        standard_convert.sample(&pos),
                        owned_convert.sample_mut(&pos, &FakeEnvironment {})
                    );
                }
            }
        }
    }
}
