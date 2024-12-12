use std::{marker::PhantomData, sync::Arc};

use enum_dispatch::enum_dispatch;

use crate::world_gen::noise::lerp;

use super::{
    component_functions::{
        ApplierImpl, ComponentFunctionImpl, ComponentReference, ComponentReferenceImplementation,
        ConverterEnvironment, ConverterImpl, DensityFunctionEnvironment, EnvironmentApplierImpl,
        ImmutableComponentFunctionImpl, MutableComponentFunctionImpl, MutableComponentReference,
        OwnedConverterEnvironment, SharedComponentReference, SharedConverterEnvironment,
    },
    NoisePos,
};

//TODO : De-duplicate code with traits and generics

#[derive(Clone)]
pub enum ImmutableValue {
    Spline(ImmutableSplineRef),
    Fixed(f32),
}

impl ImmutableValue {
    fn sample(&self, pos: &NoisePos) -> f32 {
        match self {
            Self::Fixed(val) => *val,
            Self::Spline(spline) => spline.0.sample(pos),
        }
    }
}

#[enum_dispatch(SplinePointImpl<E>)]
pub(crate) enum SplinePoint<E: DensityFunctionEnvironment> {
    Mutable(MutablePoint<E>),
    Immutable(ImmutablePoint),
}

#[enum_dispatch]
trait SplinePointImpl<E: DensityFunctionEnvironment> {
    fn sample_outside_range_mut(&mut self, oob_loc: f32, pos: &NoisePos, env: &E) -> f32;

    fn sampled_value_mut(&mut self, pos: &NoisePos, env: &E) -> f32;

    fn location(&self) -> f32;

    fn derivative(&self) -> f32;

    fn convert(self, converter: &mut dyn ConverterImpl<E>) -> SplinePoint<E>;

    fn clone_to_new_point(&self) -> SplinePoint<E>;
}

#[derive(Clone)]
pub struct ImmutablePoint {
    pub(crate) location: f32,
    pub(crate) value: ImmutableValue,
    pub(crate) derivative: f32,
}

impl ImmutablePoint {
    fn sample_outside_range(&self, oob_loc: f32, pos: &NoisePos) -> f32 {
        let oob_value = self.value.sample(pos);
        if self.derivative == 0f32 {
            oob_value
        } else {
            self.derivative * (oob_loc - self.location) + oob_value
        }
    }

    fn maybe_convert<E: DensityFunctionEnvironment>(
        &self,
        converter: &mut dyn ConverterImpl<E>,
    ) -> Option<SplinePoint<E>> {
        match &self.value {
            ImmutableValue::Fixed(_value) => None,
            ImmutableValue::Spline(spline_ref) => {
                spline_ref
                    .maybe_convert(converter)
                    .map(|converted_ref| match converted_ref {
                        SplineRef::Immutable(spline) => SplinePoint::Immutable(ImmutablePoint {
                            location: self.location,
                            value: ImmutableValue::Spline(spline),
                            derivative: self.derivative,
                        }),
                        SplineRef::Mutable(spline) => SplinePoint::Mutable(MutablePoint {
                            location: self.location,
                            value: spline,
                            derivative: self.derivative,
                        }),
                    })
            }
        }
    }
}

impl<E: DensityFunctionEnvironment> SplinePointImpl<E> for ImmutablePoint {
    #[inline]
    fn sample_outside_range_mut(&mut self, oob_loc: f32, pos: &NoisePos, _env: &E) -> f32 {
        self.sample_outside_range(oob_loc, pos)
    }

    #[inline]
    fn sampled_value_mut(&mut self, pos: &NoisePos, _env: &E) -> f32 {
        self.value.sample(pos)
    }

    #[inline]
    fn location(&self) -> f32 {
        self.location
    }

    #[inline]
    fn derivative(&self) -> f32 {
        self.derivative
    }

    fn convert(self, converter: &mut dyn ConverterImpl<E>) -> SplinePoint<E> {
        self.maybe_convert(converter)
            .unwrap_or_else(|| match self.value {
                ImmutableValue::Fixed(value) => SplinePoint::Immutable(ImmutablePoint {
                    location: self.location,
                    value: ImmutableValue::Fixed(value),
                    derivative: self.derivative,
                }),
                ImmutableValue::Spline(spline) => SplinePoint::Immutable(ImmutablePoint {
                    location: self.location,
                    value: ImmutableValue::Spline(spline),
                    derivative: self.derivative,
                }),
            })
    }

    fn clone_to_new_point(&self) -> SplinePoint<E> {
        SplinePoint::Immutable(self.clone())
    }
}

pub struct MutablePoint<E: DensityFunctionEnvironment> {
    pub(crate) location: f32,
    pub(crate) value: MutableSplineRef<E>,
    pub(crate) derivative: f32,
}

impl<E: DensityFunctionEnvironment> SplinePointImpl<E> for MutablePoint<E> {
    fn sample_outside_range_mut(&mut self, oob_loc: f32, pos: &NoisePos, env: &E) -> f32 {
        let oob_value = self.value.sample_mut(pos, env);
        if self.derivative == 0f32 {
            oob_value
        } else {
            self.derivative * (oob_loc - self.location) + oob_value
        }
    }

    #[inline]
    fn sampled_value_mut(&mut self, pos: &NoisePos, env: &E) -> f32 {
        self.value.sample_mut(pos, env)
    }

    #[inline]
    fn derivative(&self) -> f32 {
        self.derivative
    }

    #[inline]
    fn location(&self) -> f32 {
        self.location
    }

    fn convert(self, converter: &mut dyn ConverterImpl<E>) -> SplinePoint<E> {
        match self.value.convert(converter) {
            SplineRef::Immutable(spline) => SplinePoint::Immutable(ImmutablePoint {
                location: self.location,
                value: ImmutableValue::Spline(spline),
                derivative: self.derivative,
            }),
            SplineRef::Mutable(spline) => SplinePoint::Mutable(MutablePoint {
                location: self.location,
                value: spline,
                derivative: self.derivative,
            }),
        }
    }

    fn clone_to_new_point(&self) -> SplinePoint<E> {
        match self.value.clone_to_new_ref() {
            SplineRef::Immutable(spline) => SplinePoint::Immutable(ImmutablePoint {
                location: self.location,
                value: ImmutableValue::Spline(spline),
                derivative: self.derivative,
            }),
            SplineRef::Mutable(spline) => SplinePoint::Mutable(MutablePoint {
                location: self.location,
                value: spline,
                derivative: self.derivative,
            }),
        }
    }
}

/// Returns the smallest usize between min..max that does not match the predicate
fn binary_walk(min: usize, max: usize, pred: impl Fn(usize) -> bool) -> usize {
    let mut i = max - min;
    let mut min = min;
    while i > 0 {
        let j = i / 2;
        let k = min + j;
        if pred(k) {
            i = j;
        } else {
            min = k + 1;
            i -= j + 1;
        }
    }
    min
}

enum Range {
    In(usize),
    Below,
}

pub struct ImmutableSpline {
    pub(crate) function: SharedComponentReference,
    pub(crate) points: Box<[ImmutablePoint]>,
}

impl From<ImmutableSpline> for ImmutableSplineRef {
    fn from(value: ImmutableSpline) -> Self {
        Self(Arc::new(value))
    }
}

impl ImmutableSpline {
    // TODO: Is there a way to do this on the slice itself?
    fn find_index_for_location(&self, loc: f32) -> Range {
        let index_greater_than_x =
            binary_walk(0, self.points.len(), |i| loc < self.points[i].location);
        if index_greater_than_x == 0 {
            Range::Below
        } else {
            Range::In(index_greater_than_x - 1)
        }
    }

    pub fn sample(&self, pos: &NoisePos) -> f32 {
        let location = self.function.sample(pos) as f32;
        match self.find_index_for_location(location) {
            Range::In(index) => {
                if index == self.points.len() - 1 {
                    self.points[index].sample_outside_range(location, pos)
                } else {
                    let lower_point = &self.points[index];
                    let upper_point = &self.points[index + 1];

                    let lower_value = lower_point.value.sample(pos);
                    let upper_value = upper_point.value.sample(pos);

                    let x_scale = (location - lower_point.location)
                        / (upper_point.location - lower_point.location);
                    let extrapolated_lower_value = lower_point.derivative
                        * (upper_point.location - lower_point.location)
                        - (upper_value - lower_value);
                    let extrapolated_upper_value = -upper_point.derivative
                        * (upper_point.location - lower_point.location)
                        + (upper_value - lower_value);

                    (x_scale * (1f32 - x_scale))
                        * lerp(x_scale, extrapolated_lower_value, extrapolated_upper_value)
                        + lerp(x_scale, lower_value, upper_value)
                }
            }
            Range::Below => self.points[0].sample_outside_range(location, pos),
        }
    }
}

pub trait MutableSplineImpl<E: DensityFunctionEnvironment>: Send + Sync {
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f32;

    fn convert(self: Box<Self>, converter: &mut dyn ConverterImpl<E>) -> SplineRef<E>;

    fn clone_to_new_ref(&self) -> SplineRef<E>;
}

pub struct MutableSpline<E: DensityFunctionEnvironment, R: ComponentReference<E>> {
    pub(crate) function: R,
    pub(crate) points: Box<[SplinePoint<E>]>,
    _dummy: PhantomData<E>,
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> MutableSpline<E, R> {
    fn create_new_spline(
        converted_base: ComponentReferenceImplementation<E>,
        converted_points: Vec<SplinePoint<E>>,
    ) -> SplineRef<E> {
        match converted_base {
            ComponentReferenceImplementation::Shared(shared) => {
                if converted_points
                    .iter()
                    .all(|point| matches!(point, SplinePoint::Immutable(_)))
                {
                    let immutable_points: Vec<ImmutablePoint> = converted_points
                        .into_iter()
                        .map(|point| match point {
                            SplinePoint::Immutable(point) => point,
                            _ => unreachable!(),
                        })
                        .collect();
                    SplineRef::Immutable(
                        ImmutableSpline {
                            function: shared,
                            points: immutable_points.into_boxed_slice(),
                        }
                        .into(),
                    )
                } else {
                    SplineRef::Mutable(
                        MutableSpline {
                            function: shared,
                            points: converted_points.into_boxed_slice(),
                            _dummy: PhantomData::<E> {},
                        }
                        .into(),
                    )
                }
            }
            ComponentReferenceImplementation::Mutable(owned) => SplineRef::Mutable(
                MutableSpline {
                    function: owned,
                    points: converted_points.into_boxed_slice(),
                    _dummy: PhantomData::<E> {},
                }
                .into(),
            ),
        }
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> From<MutableSpline<E, R>>
    for MutableSplineRef<E>
{
    fn from(value: MutableSpline<E, R>) -> Self {
        Self(Box::new(value))
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> MutableSplineImpl<E>
    for MutableSpline<E, R>
{
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f32 {
        let location = self.function.sample_mut(pos, env) as f32;
        match self.find_index_for_location(location) {
            Range::In(index) => {
                if index == self.points.len() - 1 {
                    self.points[index].sample_outside_range_mut(location, pos, env)
                } else {
                    let lower_value = self.points[index].sampled_value_mut(pos, env);
                    let upper_value = self.points[index + 1].sampled_value_mut(pos, env);

                    let lower_point = &self.points[index];
                    let upper_point = &self.points[index + 1];

                    let x_scale = (location - lower_point.location())
                        / (upper_point.location() - lower_point.location());
                    let extrapolated_lower_value = lower_point.derivative()
                        * (upper_point.location() - lower_point.location())
                        - (upper_value - lower_value);
                    let extrapolated_upper_value = -upper_point.derivative()
                        * (upper_point.location() - lower_point.location())
                        + (upper_value - lower_value);

                    (x_scale * (1f32 - x_scale))
                        * lerp(x_scale, extrapolated_lower_value, extrapolated_upper_value)
                        + lerp(x_scale, lower_value, upper_value)
                }
            }
            Range::Below => self.points[0].sample_outside_range_mut(location, pos, env),
        }
    }

    fn convert(self: Box<Self>, converter: &mut dyn ConverterImpl<E>) -> SplineRef<E> {
        let converted_base = self.function.convert(converter);
        let points = self.points.into_vec();
        let converted_points = points
            .into_iter()
            .map(|point| point.convert(converter))
            .collect();

        Self::create_new_spline(converted_base, converted_points)
    }

    fn clone_to_new_ref(&self) -> SplineRef<E> {
        let cloned_function = self.function.clone_to_new_ref();
        let cloned_points = self
            .points
            .iter()
            .map(|point| point.clone_to_new_point())
            .collect();

        Self::create_new_spline(cloned_function, cloned_points)
    }
}

impl<E: DensityFunctionEnvironment, R: ComponentReference<E>> MutableSpline<E, R> {
    fn find_index_for_location(&self, loc: f32) -> Range {
        let index_greater_than_x =
            binary_walk(0, self.points.len(), |i| loc < self.points[i].location());
        if index_greater_than_x == 0 {
            Range::Below
        } else {
            Range::In(index_greater_than_x - 1)
        }
    }
}

pub(crate) enum SplineRef<E: DensityFunctionEnvironment> {
    Immutable(ImmutableSplineRef),
    Mutable(MutableSplineRef<E>),
}

pub(crate) trait SplineRefImpl<E: DensityFunctionEnvironment>: Send + Sync {
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f32;

    fn convert(self, converter: &mut dyn ConverterImpl<E>) -> SplineRef<E>;

    fn clone_to_new_ref(&self) -> SplineRef<E>;

    fn into_ref(self) -> SplineRef<E>;
}

pub(crate) struct MutableSplineRef<E: DensityFunctionEnvironment>(Box<dyn MutableSplineImpl<E>>);

impl<E: DensityFunctionEnvironment> SplineRefImpl<E> for MutableSplineRef<E> {
    #[inline]
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f32 {
        self.0.sample_mut(pos, env)
    }

    #[inline]
    fn convert(self, converter: &mut dyn ConverterImpl<E>) -> SplineRef<E> {
        self.0.convert(converter)
    }

    fn clone_to_new_ref(&self) -> SplineRef<E> {
        self.0.clone_to_new_ref()
    }

    fn into_ref(self) -> SplineRef<E> {
        SplineRef::Mutable(self)
    }
}

#[derive(Clone)]
pub(crate) struct ImmutableSplineRef(Arc<ImmutableSpline>);

impl ImmutableSplineRef {
    pub fn maybe_convert<E: DensityFunctionEnvironment>(
        &self,
        converter: &mut dyn ConverterImpl<E>,
    ) -> Option<SplineRef<E>> {
        let converted_base = self.0.function.maybe_convert(converter);
        let maybe_converted_points: Vec<Option<SplinePoint<E>>> = self
            .0
            .points
            .iter()
            .map(|point| point.maybe_convert(converter))
            .collect();

        if converted_base.is_none() && maybe_converted_points.iter().all(|point| point.is_none()) {
            None
        } else {
            let converted_base = converted_base.unwrap_or_else(|| self.0.function.clone().into());
            let converted_points: Vec<SplinePoint<E>> = maybe_converted_points
                .into_iter()
                .enumerate()
                .map(|(index, point)| {
                    if let Some(point) = point {
                        point
                    } else {
                        self.0.points[index].clone().into()
                    }
                })
                .collect();

            Some(match converted_base {
                ComponentReferenceImplementation::Shared(shared) => {
                    if converted_points
                        .iter()
                        .all(|point| matches!(point, SplinePoint::Immutable(_)))
                    {
                        let immutable_points: Vec<ImmutablePoint> = converted_points
                            .into_iter()
                            .map(|point| match point {
                                SplinePoint::Immutable(point) => point,
                                _ => unreachable!(),
                            })
                            .collect();
                        SplineRef::Immutable(
                            ImmutableSpline {
                                function: shared,
                                points: immutable_points.into_boxed_slice(),
                            }
                            .into(),
                        )
                    } else {
                        SplineRef::Mutable(
                            MutableSpline {
                                function: shared,
                                points: converted_points.into_boxed_slice(),
                                _dummy: PhantomData::<E> {},
                            }
                            .into(),
                        )
                    }
                }
                ComponentReferenceImplementation::Mutable(owned) => SplineRef::Mutable(
                    MutableSpline {
                        function: owned,
                        points: converted_points.into_boxed_slice(),
                        _dummy: PhantomData::<E> {},
                    }
                    .into(),
                ),
            })
        }
    }
}

impl<E: DensityFunctionEnvironment> SplineRefImpl<E> for ImmutableSplineRef {
    #[inline]
    fn sample_mut(&mut self, pos: &NoisePos, _env: &E) -> f32 {
        self.0.sample(pos)
    }

    fn convert(self, converter: &mut dyn ConverterImpl<E>) -> SplineRef<E> {
        self.maybe_convert(converter)
            .unwrap_or_else(|| SplineRef::Immutable(self.clone()))
    }

    fn clone_to_new_ref(&self) -> SplineRef<E> {
        SplineRef::Immutable(self.clone())
    }

    fn into_ref(self) -> SplineRef<E> {
        SplineRef::Immutable(self)
    }
}

#[derive(Clone)]
pub struct SplineFunction<E: DensityFunctionEnvironment, S: SplineRefImpl<E>> {
    pub(crate) spline: S,
    _dummy: PhantomData<E>,
}

impl<E: DensityFunctionEnvironment, S: SplineRefImpl<E>> SplineFunction<E, S> {
    pub fn new(spline: S) -> Self {
        Self {
            spline,
            _dummy: PhantomData::<E> {},
        }
    }

    pub fn create_new_ref(spline: SplineRef<E>) -> ComponentReferenceImplementation<E> {
        match spline {
            SplineRef::Mutable(mutable_spline) => ComponentReferenceImplementation::Mutable(
                SplineFunction {
                    spline: mutable_spline,
                    _dummy: PhantomData::<E> {},
                }
                .into(),
            ),
            SplineRef::Immutable(immutable_spline) => ComponentReferenceImplementation::Shared(
                SplineFunction::<E, ImmutableSplineRef> {
                    spline: immutable_spline,
                    _dummy: PhantomData::<E> {},
                }
                .into(),
            ),
        }
    }
}

impl<E: DensityFunctionEnvironment, S: SplineRefImpl<E> + 'static> From<SplineFunction<E, S>>
    for MutableComponentReference<E>
{
    fn from(value: SplineFunction<E, S>) -> Self {
        Self(Box::new(value))
    }
}

impl<E: DensityFunctionEnvironment, S: SplineRefImpl<E>> ComponentFunctionImpl
    for SplineFunction<E, S>
{
}

impl<E: DensityFunctionEnvironment, S: SplineRefImpl<E>> MutableComponentFunctionImpl<E>
    for SplineFunction<E, S>
{
    #[inline]
    fn sample_mut(&mut self, pos: &NoisePos, env: &E) -> f64 {
        self.spline.sample_mut(pos, env) as f64
    }

    #[inline]
    fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>) {
        applier.fill_mut(arr, self);
    }

    fn environment(&self) -> ConverterEnvironment<E> {
        ConverterEnvironment::Spline(&self.spline)
    }

    fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<E> {
        OwnedConverterEnvironment::Spline(self.spline.into_ref())
    }

    fn convert(
        self: Box<Self>,
        converter: &mut dyn ConverterImpl<E>,
    ) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(self.spline.convert(converter))
    }

    fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E> {
        Self::create_new_ref(self.spline.clone_to_new_ref())
    }
}

impl<E: DensityFunctionEnvironment> ImmutableComponentFunctionImpl
    for SplineFunction<E, ImmutableSplineRef>
{
    #[inline]
    fn sample(&self, pos: &NoisePos) -> f64 {
        self.spline.0.sample(pos) as f64
    }

    #[inline]
    fn fill(&self, arr: &mut [f64], applier: &mut dyn ApplierImpl) {
        applier.fill(arr, self);
    }

    fn shared_environment(&self) -> SharedConverterEnvironment {
        SharedConverterEnvironment::Spline(&self.spline)
    }
}

#[derive(Clone, Copy)]
pub enum FloatAmplifier {
    Identity,
    OffsetAmplifier,
    FactorAmplifier,
    JaggednessAmplifier,
}

impl FloatAmplifier {
    #[inline]
    pub fn apply(&self, f: f32) -> f32 {
        match self {
            Self::Identity => f,
            Self::OffsetAmplifier => {
                if f < 0f32 {
                    f
                } else {
                    f * 2f32
                }
            }
            Self::FactorAmplifier => 1.25f32 - 6.25f32 / (f + 5f32),
            Self::JaggednessAmplifier => f * 2f32,
        }
    }
}

pub struct SplineBuilder {
    function: SharedComponentReference,
    amplifier: FloatAmplifier,
    points: Vec<ImmutablePoint>,
}

impl SplineBuilder {
    pub fn new(function: SharedComponentReference, amplifier: FloatAmplifier) -> Self {
        Self {
            function,
            amplifier,
            points: Vec::new(),
        }
    }

    #[must_use]
    #[inline]
    pub fn add_fixed_value(self, location: f32, value: f32) -> Self {
        self.add_fixed_value_derivative(location, value, 0f32)
    }

    #[must_use]
    pub fn add_fixed_value_derivative(self, location: f32, value: f32, derivative: f32) -> Self {
        let amplified = self.amplifier.apply(value);
        self.add_value(location, ImmutableValue::Fixed(amplified), derivative)
    }

    #[must_use]
    #[inline]
    pub fn add_spline_value(self, location: f32, value: ImmutableSplineRef) -> Self {
        self.add_spline_value_derivative(location, value, 0f32)
    }

    #[must_use]
    pub fn add_spline_value_derivative(
        self,
        location: f32,
        value: ImmutableSplineRef,
        derivative: f32,
    ) -> Self {
        self.add_value(location, ImmutableValue::Spline(value), derivative)
    }

    #[must_use]
    pub fn add_value(mut self, location: f32, value: ImmutableValue, derivative: f32) -> Self {
        #[cfg(debug_assertions)]
        if let Some(last) = self.points.last() {
            assert!(location > last.location);
        }

        self.points.push(ImmutablePoint {
            location,
            value,
            derivative,
        });

        self
    }

    pub fn build(self) -> ImmutableSpline {
        ImmutableSpline {
            function: self.function,
            points: self.points.into_boxed_slice(),
        }
    }
}

#[cfg(test)]
mod test {

    use pumpkin_core::random::{legacy_rand::LegacyRand, RandomDeriver, RandomImpl};

    use crate::world_gen::noise::density::{
        built_in_density_function::CONTINENTS_OVERWORLD,
        component_functions::{ComponentReference, NoEnvironment, SharedComponentReference},
        test::{FakeEnvironment, OwnedConverter, TestConverter},
        NoisePos, UnblendedNoisePos,
    };

    use super::{FloatAmplifier, ImmutableSplineRef, SplineBuilder, SplineFunction};

    #[test]
    fn test_correctness() {
        let mut rand = LegacyRand::from_seed(0);
        let deriver = rand.next_splitter();
        let mut converter = TestConverter {
            splitter: RandomDeriver::Legacy(deriver),
        };

        let spline = SplineBuilder::new(
            CONTINENTS_OVERWORLD
                .clone()
                .convert(&mut converter)
                .assert_shared(),
            FloatAmplifier::Identity,
        )
        .add_fixed_value_derivative(-1.1f32, 0.044f32, -0.1f32)
        .add_fixed_value(-1.02f32, -0.2222f32)
        .add_fixed_value_derivative(-0.51f32, -0.2222f32, 0.1f32)
        .add_fixed_value(-0.44f32, -0.12f32)
        .add_fixed_value_derivative(-0.18f32, -0.12f32, 0.1f32)
        .build();

        let values = [
            ((-100, -100), -0.07604788f32),
            ((-100, -90), -0.07773465f32),
            ((-100, -80), -0.07928875f32),
            ((-100, -70), -0.08118123f32),
            ((-100, -60), -0.08313452f32),
            ((-100, -50), -0.083534524f32),
            ((-100, -40), -0.086245626f32),
            ((-100, -30), -0.08444518f32),
            ((-100, -20), -0.08520311f32),
            ((-100, -10), -0.08629203f32),
            ((-100, 0), -0.08723046f32),
            ((-100, 10), -0.0888218f32),
            ((-100, 20), -0.09126012f32),
            ((-100, 30), -0.092776805f32),
            ((-100, 40), -0.09374735f32),
            ((-100, 50), -0.09605039f32),
            ((-100, 60), -0.09593062f32),
            ((-100, 70), -0.09638955f32),
            ((-100, 80), -0.09660137f32),
            ((-100, 90), -0.09732263f32),
            ((-100, 100), -0.09875606f32),
            ((-90, -100), -0.07605396f32),
            ((-90, -90), -0.07945493f32),
            ((-90, -80), -0.08126007f32),
            ((-90, -70), -0.0827491f32),
            ((-90, -60), -0.084900096f32),
            ((-90, -50), -0.087383136f32),
            ((-90, -40), -0.08763948f32),
            ((-90, -30), -0.08750856f32),
            ((-90, -20), -0.08923715f32),
            ((-90, -10), -0.08950907f32),
            ((-90, 0), -0.08966042f32),
            ((-90, 10), -0.091661744f32),
            ((-90, 20), -0.09423652f32),
            ((-90, 30), -0.09460543f32),
            ((-90, 40), -0.09597071f32),
            ((-90, 50), -0.09838261f32),
            ((-90, 60), -0.09757312f32),
            ((-90, 70), -0.09852694f32),
            ((-90, 80), -0.09875368f32),
            ((-90, 90), -0.10035795f32),
            ((-90, 100), -0.099384755f32),
            ((-80, -100), -0.07845141f32),
            ((-80, -90), -0.0802805f32),
            ((-80, -80), -0.08364986f32),
            ((-80, -70), -0.085202575f32),
            ((-80, -60), -0.0893721f32),
            ((-80, -50), -0.09021371f32),
            ((-80, -40), -0.088948175f32),
            ((-80, -30), -0.09102255f32),
            ((-80, -20), -0.09252357f32),
            ((-80, -10), -0.092946626f32),
            ((-80, 0), -0.0917982f32),
            ((-80, 10), -0.09275723f32),
            ((-80, 20), -0.09508084f32),
            ((-80, 30), -0.09741648f32),
            ((-80, 40), -0.09864242f32),
            ((-80, 50), -0.1010126f32),
            ((-80, 60), -0.10251929f32),
            ((-80, 70), -0.10301858f32),
            ((-80, 80), -0.10152783f32),
            ((-80, 90), -0.1018514f32),
            ((-80, 100), -0.101050965f32),
            ((-70, -100), -0.08047484f32),
            ((-70, -90), -0.0827865f32),
            ((-70, -80), -0.08533125f32),
            ((-70, -70), -0.087316774f32),
            ((-70, -60), -0.08924434f32),
            ((-70, -50), -0.09139694f32),
            ((-70, -40), -0.092277184f32),
            ((-70, -30), -0.09338808f32),
            ((-70, -20), -0.095891915f32),
            ((-70, -10), -0.095507845f32),
            ((-70, 0), -0.09539513f32),
            ((-70, 10), -0.0951063f32),
            ((-70, 20), -0.09744669f32),
            ((-70, 30), -0.101334676f32),
            ((-70, 40), -0.10026048f32),
            ((-70, 50), -0.102552444f32),
            ((-70, 60), -0.104417525f32),
            ((-70, 70), -0.10407996f32),
            ((-70, 80), -0.10480991f32),
            ((-70, 90), -0.10390431f32),
            ((-70, 100), -0.102846265f32),
            ((-60, -100), -0.08198626f32),
            ((-60, -90), -0.08353661f32),
            ((-60, -80), -0.08603583f32),
            ((-60, -70), -0.08766833f32),
            ((-60, -60), -0.089199014f32),
            ((-60, -50), -0.091917805f32),
            ((-60, -40), -0.09299549f32),
            ((-60, -30), -0.0956157f32),
            ((-60, -20), -0.09864686f32),
            ((-60, -10), -0.09914974f32),
            ((-60, 0), -0.10039974f32),
            ((-60, 10), -0.10030475f32),
            ((-60, 20), -0.10159342f32),
            ((-60, 30), -0.10401981f32),
            ((-60, 40), -0.10531161f32),
            ((-60, 50), -0.10649038f32),
            ((-60, 60), -0.10697486f32),
            ((-60, 70), -0.10814563f32),
            ((-60, 80), -0.10756317f32),
            ((-60, 90), -0.10590332f32),
            ((-60, 100), -0.10525697f32),
            ((-50, -100), -0.08205913f32),
            ((-50, -90), -0.08393769f32),
            ((-50, -80), -0.086751714f32),
            ((-50, -70), -0.089658506f32),
            ((-50, -60), -0.092683166f32),
            ((-50, -50), -0.09480399f32),
            ((-50, -40), -0.09606384f32),
            ((-50, -30), -0.09774114f32),
            ((-50, -20), -0.100446284f32),
            ((-50, -10), -0.10084822f32),
            ((-50, 0), -0.10314222f32),
            ((-50, 10), -0.10293749f32),
            ((-50, 20), -0.10586517f32),
            ((-50, 30), -0.10697569f32),
            ((-50, 40), -0.10719836f32),
            ((-50, 50), -0.10689005f32),
            ((-50, 60), -0.10781257f32),
            ((-50, 70), -0.10913751f32),
            ((-50, 80), -0.111755826f32),
            ((-50, 90), -0.10983417f32),
            ((-50, 100), -0.10947363f32),
            ((-40, -100), -0.0824882f32),
            ((-40, -90), -0.08360703f32),
            ((-40, -80), -0.086381115f32),
            ((-40, -70), -0.09064654f32),
            ((-40, -60), -0.093374886f32),
            ((-40, -50), -0.09642702f32),
            ((-40, -40), -0.09817896f32),
            ((-40, -30), -0.100624494f32),
            ((-40, -20), -0.10402103f32),
            ((-40, -10), -0.103324674f32),
            ((-40, 0), -0.10734479f32),
            ((-40, 10), -0.10787663f32),
            ((-40, 20), -0.11011673f32),
            ((-40, 30), -0.10879173f32),
            ((-40, 40), -0.10899488f32),
            ((-40, 50), -0.1093022f32),
            ((-40, 60), -0.11100974f32),
            ((-40, 70), -0.11430037f32),
            ((-40, 80), -0.11370994f32),
            ((-40, 90), -0.1117007f32),
            ((-40, 100), -0.111225165f32),
            ((-30, -100), -0.08073887f32),
            ((-30, -90), -0.08562371f32),
            ((-30, -80), -0.08811793f32),
            ((-30, -70), -0.092022136f32),
            ((-30, -60), -0.09238706f32),
            ((-30, -50), -0.09777247f32),
            ((-30, -40), -0.10047619f32),
            ((-30, -30), -0.1017582f32),
            ((-30, -20), -0.10325858f32),
            ((-30, -10), -0.10587716f32),
            ((-30, 0), -0.10807945f32),
            ((-30, 10), -0.11133594f32),
            ((-30, 20), -0.11149085f32),
            ((-30, 30), -0.11169845f32),
            ((-30, 40), -0.11191458f32),
            ((-30, 50), -0.11245423f32),
            ((-30, 60), -0.11395778f32),
            ((-30, 70), -0.116144754f32),
            ((-30, 80), -0.115703195f32),
            ((-30, 90), -0.11417798f32),
            ((-30, 100), -0.113027185f32),
            ((-20, -100), -0.081831336f32),
            ((-20, -90), -0.08537665f32),
            ((-20, -80), -0.08823441f32),
            ((-20, -70), -0.09004638f32),
            ((-20, -60), -0.091682106f32),
            ((-20, -50), -0.09560484f32),
            ((-20, -40), -0.100073636f32),
            ((-20, -30), -0.102195345f32),
            ((-20, -20), -0.103985146f32),
            ((-20, -10), -0.105535336f32),
            ((-20, 0), -0.11013269f32),
            ((-20, 10), -0.1115511f32),
            ((-20, 20), -0.113062836f32),
            ((-20, 30), -0.112847894f32),
            ((-20, 40), -0.11428483f32),
            ((-20, 50), -0.11667834f32),
            ((-20, 60), -0.117276795f32),
            ((-20, 70), -0.11689238f32),
            ((-20, 80), -0.1160782f32),
            ((-20, 90), -0.11455002f32),
            ((-20, 100), -0.11329481f32),
            ((-10, -100), -0.08257191f32),
            ((-10, -90), -0.085004464f32),
            ((-10, -80), -0.08695547f32),
            ((-10, -70), -0.08808699f32),
            ((-10, -60), -0.091026604f32),
            ((-10, -50), -0.09544293f32),
            ((-10, -40), -0.09790615f32),
            ((-10, -30), -0.10184401f32),
            ((-10, -20), -0.10396968f32),
            ((-10, -10), -0.1076317f32),
            ((-10, 0), -0.11115202f32),
            ((-10, 10), -0.11212789f32),
            ((-10, 20), -0.11348673f32),
            ((-10, 30), -0.11471321f32),
            ((-10, 40), -0.114718616f32),
            ((-10, 50), -0.11725365f32),
            ((-10, 60), -0.11695717f32),
            ((-10, 70), -0.116616994f32),
            ((-10, 80), -0.11536371f32),
            ((-10, 90), -0.11446484f32),
            ((-10, 100), -0.11369774f32),
            ((0, -100), -0.08459158f32),
            ((0, -90), -0.08656791f32),
            ((0, -80), -0.08754034f32),
            ((0, -70), -0.08878641f32),
            ((0, -60), -0.09282567f32),
            ((0, -50), -0.09490024f32),
            ((0, -40), -0.09933582f32),
            ((0, -30), -0.10083613f32),
            ((0, -20), -0.104990706f32),
            ((0, -10), -0.10789699f32),
            ((0, 0), -0.10978986f32),
            ((0, 10), -0.11214093f32),
            ((0, 20), -0.11311828f32),
            ((0, 30), -0.11421281f32),
            ((0, 40), -0.11489451f32),
            ((0, 50), -0.11654575f32),
            ((0, 60), -0.116693474f32),
            ((0, 70), -0.11676903f32),
            ((0, 80), -0.11486762f32),
            ((0, 90), -0.11349271f32),
            ((0, 100), -0.11301751f32),
            ((10, -100), -0.08532186f32),
            ((10, -90), -0.087390676f32),
            ((10, -80), -0.08947607f32),
            ((10, -70), -0.091350235f32),
            ((10, -60), -0.092257306f32),
            ((10, -50), -0.094102554f32),
            ((10, -40), -0.09877145f32),
            ((10, -30), -0.101968475f32),
            ((10, -20), -0.10476847f32),
            ((10, -10), -0.10820749f32),
            ((10, 0), -0.1099116f32),
            ((10, 10), -0.1106353f32),
            ((10, 20), -0.11173092f32),
            ((10, 30), -0.11349803f32),
            ((10, 40), -0.11299823f32),
            ((10, 50), -0.11539698f32),
            ((10, 60), -0.11715141f32),
            ((10, 70), -0.11558097f32),
            ((10, 80), -0.114774175f32),
            ((10, 90), -0.11429333f32),
            ((10, 100), -0.112418234f32),
            ((20, -100), -0.08536324f32),
            ((20, -90), -0.08702316f32),
            ((20, -80), -0.09097032f32),
            ((20, -70), -0.09171111f32),
            ((20, -60), -0.092209786f32),
            ((20, -50), -0.09390856f32),
            ((20, -40), -0.09674665f32),
            ((20, -30), -0.0995738f32),
            ((20, -20), -0.10170178f32),
            ((20, -10), -0.107144f32),
            ((20, 0), -0.10934077f32),
            ((20, 10), -0.11114335f32),
            ((20, 20), -0.11120629f32),
            ((20, 30), -0.11104426f32),
            ((20, 40), -0.109872885f32),
            ((20, 50), -0.11435944f32),
            ((20, 60), -0.11630697f32),
            ((20, 70), -0.11403515f32),
            ((20, 80), -0.11370773f32),
            ((20, 90), -0.11165723f32),
            ((20, 100), -0.11122058f32),
            ((30, -100), -0.083562575f32),
            ((30, -90), -0.08608784f32),
            ((30, -80), -0.08794823f32),
            ((30, -70), -0.091189116f32),
            ((30, -60), -0.093153656f32),
            ((30, -50), -0.0951614f32),
            ((30, -40), -0.0953993f32),
            ((30, -30), -0.096983574f32),
            ((30, -20), -0.10012217f32),
            ((30, -10), -0.10612148f32),
            ((30, 0), -0.10943102f32),
            ((30, 10), -0.10920269f32),
            ((30, 20), -0.10791202f32),
            ((30, 30), -0.10835938f32),
            ((30, 40), -0.10786465f32),
            ((30, 50), -0.11130254f32),
            ((30, 60), -0.1126149f32),
            ((30, 70), -0.110773936f32),
            ((30, 80), -0.109979525f32),
            ((30, 90), -0.10987309f32),
            ((30, 100), -0.108747214f32),
            ((40, -100), -0.08093807f32),
            ((40, -90), -0.08576739f32),
            ((40, -80), -0.085233085f32),
            ((40, -70), -0.09012596f32),
            ((40, -60), -0.09217769f32),
            ((40, -50), -0.09351354f32),
            ((40, -40), -0.09589194f32),
            ((40, -30), -0.09674299f32),
            ((40, -20), -0.09954912f32),
            ((40, -10), -0.10170849f32),
            ((40, 0), -0.10476398f32),
            ((40, 10), -0.10546719f32),
            ((40, 20), -0.10537742f32),
            ((40, 30), -0.105657674f32),
            ((40, 40), -0.10615531f32),
            ((40, 50), -0.10808634f32),
            ((40, 60), -0.10634413f32),
            ((40, 70), -0.107364416f32),
            ((40, 80), -0.10856771f32),
            ((40, 90), -0.10862812f32),
            ((40, 100), -0.10680454f32),
            ((50, -100), -0.082533084f32),
            ((50, -90), -0.086810954f32),
            ((50, -80), -0.085271746f32),
            ((50, -70), -0.08944471f32),
            ((50, -60), -0.09193231f32),
            ((50, -50), -0.09362271f32),
            ((50, -40), -0.09453575f32),
            ((50, -30), -0.095691115f32),
            ((50, -20), -0.098154165f32),
            ((50, -10), -0.097271696f32),
            ((50, 0), -0.09942495f32),
            ((50, 10), -0.10164021f32),
            ((50, 20), -0.10168414f32),
            ((50, 30), -0.104937024f32),
            ((50, 40), -0.10539722f32),
            ((50, 50), -0.10424481f32),
            ((50, 60), -0.10219841f32),
            ((50, 70), -0.10340266f32),
            ((50, 80), -0.106310576f32),
            ((50, 90), -0.10595322f32),
            ((50, 100), -0.10645929f32),
            ((60, -100), -0.08115639f32),
            ((60, -90), -0.08455347f32),
            ((60, -80), -0.08534711f32),
            ((60, -70), -0.08780121f32),
            ((60, -60), -0.090132974f32),
            ((60, -50), -0.091330595f32),
            ((60, -40), -0.091192245f32),
            ((60, -30), -0.0924395f32),
            ((60, -20), -0.09527585f32),
            ((60, -10), -0.09565725f32),
            ((60, 0), -0.09630064f32),
            ((60, 10), -0.09673829f32),
            ((60, 20), -0.09658762f32),
            ((60, 30), -0.09961178f32),
            ((60, 40), -0.100632355f32),
            ((60, 50), -0.10012698f32),
            ((60, 60), -0.09957688f32),
            ((60, 70), -0.10111454f32),
            ((60, 80), -0.10357678f32),
            ((60, 90), -0.104492605f32),
            ((60, 100), -0.10421045f32),
            ((70, -100), -0.074997574f32),
            ((70, -90), -0.07923891f32),
            ((70, -80), -0.08061024f32),
            ((70, -70), -0.08389824f32),
            ((70, -60), -0.08752339f32),
            ((70, -50), -0.090026386f32),
            ((70, -40), -0.09045799f32),
            ((70, -30), -0.091568656f32),
            ((70, -20), -0.09186759f32),
            ((70, -10), -0.09279721f32),
            ((70, 0), -0.093260504f32),
            ((70, 10), -0.092824616f32),
            ((70, 20), -0.093960315f32),
            ((70, 30), -0.09660603f32),
            ((70, 40), -0.09790762f32),
            ((70, 50), -0.09831002f32),
            ((70, 60), -0.09862129f32),
            ((70, 70), -0.1009444f32),
            ((70, 80), -0.101910815f32),
            ((70, 90), -0.10201355f32),
            ((70, 100), -0.1022472f32),
            ((80, -100), -0.07063179f32),
            ((80, -90), -0.073081866f32),
            ((80, -80), -0.0773198f32),
            ((80, -70), -0.079035714f32),
            ((80, -60), -0.08466645f32),
            ((80, -50), -0.08781248f32),
            ((80, -40), -0.087890774f32),
            ((80, -30), -0.09016053f32),
            ((80, -20), -0.09094465f32),
            ((80, -10), -0.09138842f32),
            ((80, 0), -0.090936236f32),
            ((80, 10), -0.09181613f32),
            ((80, 20), -0.09273602f32),
            ((80, 30), -0.09400447f32),
            ((80, 40), -0.094502196f32),
            ((80, 50), -0.09436651f32),
            ((80, 60), -0.09598513f32),
            ((80, 70), -0.098289296f32),
            ((80, 80), -0.10086883f32),
            ((80, 90), -0.101560704f32),
            ((80, 100), -0.10193728f32),
            ((90, -100), -0.07009827f32),
            ((90, -90), -0.071226865f32),
            ((90, -80), -0.07469955f32),
            ((90, -70), -0.07523824f32),
            ((90, -60), -0.07865613f32),
            ((90, -50), -0.084405504f32),
            ((90, -40), -0.085147366f32),
            ((90, -30), -0.08834492f32),
            ((90, -20), -0.08923916f32),
            ((90, -10), -0.08832547f32),
            ((90, 0), -0.087817885f32),
            ((90, 10), -0.09013721f32),
            ((90, 20), -0.091518745f32),
            ((90, 30), -0.091617286f32),
            ((90, 40), -0.0920376f32),
            ((90, 50), -0.09236775f32),
            ((90, 60), -0.094668776f32),
            ((90, 70), -0.096736684f32),
            ((90, 80), -0.099345334f32),
            ((90, 90), -0.10124628f32),
            ((90, 100), -0.10255872f32),
            ((100, -100), -0.06807774f32),
            ((100, -90), -0.07107438f32),
            ((100, -80), -0.072487816f32),
            ((100, -70), -0.075734794f32),
            ((100, -60), -0.07796392f32),
            ((100, -50), -0.08121155f32),
            ((100, -40), -0.08184202f32),
            ((100, -30), -0.08378057f32),
            ((100, -20), -0.0849825f32),
            ((100, -10), -0.08561178f32),
            ((100, 0), -0.08618045f32),
            ((100, 10), -0.08772436f32),
            ((100, 20), -0.08878901f32),
            ((100, 30), -0.08852096f32),
            ((100, 40), -0.0906315f32),
            ((100, 50), -0.091744505f32),
            ((100, 60), -0.093411185f32),
            ((100, 70), -0.09586958f32),
            ((100, 80), -0.098537765f32),
            ((100, 90), -0.10159851f32),
            ((100, 100), -0.10332601f32),
        ];

        for ((x, z), result) in values {
            let pos = &NoisePos::Unblended(UnblendedNoisePos::new(x, 60, z));
            assert_eq!(spline.sample(pos), result);
        }
    }

    #[test]
    fn test_owned_correctness() {
        let mut rand = LegacyRand::from_seed(0);
        let deriver = rand.next_splitter();
        let mut converter = OwnedConverter {
            splitter: RandomDeriver::Legacy(deriver),
        };

        let spline = SplineBuilder::new(CONTINENTS_OVERWORLD.clone(), FloatAmplifier::Identity)
            .add_fixed_value_derivative(-1.1f32, 0.044f32, -0.1f32)
            .add_fixed_value(-1.02f32, -0.2222f32)
            .add_fixed_value_derivative(-0.51f32, -0.2222f32, 0.1f32)
            .add_fixed_value(-0.44f32, -0.12f32)
            .add_fixed_value_derivative(-0.18f32, -0.12f32, 0.1f32)
            .build();

        let spline_function: SharedComponentReference =
            SplineFunction::<NoEnvironment, ImmutableSplineRef>::new(spline.into()).into();
        let mut func = spline_function.convert_to_dyn(&mut converter);

        let values = [
            ((-100, -100), -0.07604788f32),
            ((-100, -90), -0.07773465f32),
            ((-100, -80), -0.07928875f32),
            ((-100, -70), -0.08118123f32),
            ((-100, -60), -0.08313452f32),
            ((-100, -50), -0.083534524f32),
            ((-100, -40), -0.086245626f32),
            ((-100, -30), -0.08444518f32),
            ((-100, -20), -0.08520311f32),
            ((-100, -10), -0.08629203f32),
            ((-100, 0), -0.08723046f32),
            ((-100, 10), -0.0888218f32),
            ((-100, 20), -0.09126012f32),
            ((-100, 30), -0.092776805f32),
            ((-100, 40), -0.09374735f32),
            ((-100, 50), -0.09605039f32),
            ((-100, 60), -0.09593062f32),
            ((-100, 70), -0.09638955f32),
            ((-100, 80), -0.09660137f32),
            ((-100, 90), -0.09732263f32),
            ((-100, 100), -0.09875606f32),
            ((-90, -100), -0.07605396f32),
            ((-90, -90), -0.07945493f32),
            ((-90, -80), -0.08126007f32),
            ((-90, -70), -0.0827491f32),
            ((-90, -60), -0.084900096f32),
            ((-90, -50), -0.087383136f32),
            ((-90, -40), -0.08763948f32),
            ((-90, -30), -0.08750856f32),
            ((-90, -20), -0.08923715f32),
            ((-90, -10), -0.08950907f32),
            ((-90, 0), -0.08966042f32),
            ((-90, 10), -0.091661744f32),
            ((-90, 20), -0.09423652f32),
            ((-90, 30), -0.09460543f32),
            ((-90, 40), -0.09597071f32),
            ((-90, 50), -0.09838261f32),
            ((-90, 60), -0.09757312f32),
            ((-90, 70), -0.09852694f32),
            ((-90, 80), -0.09875368f32),
            ((-90, 90), -0.10035795f32),
            ((-90, 100), -0.099384755f32),
            ((-80, -100), -0.07845141f32),
            ((-80, -90), -0.0802805f32),
            ((-80, -80), -0.08364986f32),
            ((-80, -70), -0.085202575f32),
            ((-80, -60), -0.0893721f32),
            ((-80, -50), -0.09021371f32),
            ((-80, -40), -0.088948175f32),
            ((-80, -30), -0.09102255f32),
            ((-80, -20), -0.09252357f32),
            ((-80, -10), -0.092946626f32),
            ((-80, 0), -0.0917982f32),
            ((-80, 10), -0.09275723f32),
            ((-80, 20), -0.09508084f32),
            ((-80, 30), -0.09741648f32),
            ((-80, 40), -0.09864242f32),
            ((-80, 50), -0.1010126f32),
            ((-80, 60), -0.10251929f32),
            ((-80, 70), -0.10301858f32),
            ((-80, 80), -0.10152783f32),
            ((-80, 90), -0.1018514f32),
            ((-80, 100), -0.101050965f32),
            ((-70, -100), -0.08047484f32),
            ((-70, -90), -0.0827865f32),
            ((-70, -80), -0.08533125f32),
            ((-70, -70), -0.087316774f32),
            ((-70, -60), -0.08924434f32),
            ((-70, -50), -0.09139694f32),
            ((-70, -40), -0.092277184f32),
            ((-70, -30), -0.09338808f32),
            ((-70, -20), -0.095891915f32),
            ((-70, -10), -0.095507845f32),
            ((-70, 0), -0.09539513f32),
            ((-70, 10), -0.0951063f32),
            ((-70, 20), -0.09744669f32),
            ((-70, 30), -0.101334676f32),
            ((-70, 40), -0.10026048f32),
            ((-70, 50), -0.102552444f32),
            ((-70, 60), -0.104417525f32),
            ((-70, 70), -0.10407996f32),
            ((-70, 80), -0.10480991f32),
            ((-70, 90), -0.10390431f32),
            ((-70, 100), -0.102846265f32),
            ((-60, -100), -0.08198626f32),
            ((-60, -90), -0.08353661f32),
            ((-60, -80), -0.08603583f32),
            ((-60, -70), -0.08766833f32),
            ((-60, -60), -0.089199014f32),
            ((-60, -50), -0.091917805f32),
            ((-60, -40), -0.09299549f32),
            ((-60, -30), -0.0956157f32),
            ((-60, -20), -0.09864686f32),
            ((-60, -10), -0.09914974f32),
            ((-60, 0), -0.10039974f32),
            ((-60, 10), -0.10030475f32),
            ((-60, 20), -0.10159342f32),
            ((-60, 30), -0.10401981f32),
            ((-60, 40), -0.10531161f32),
            ((-60, 50), -0.10649038f32),
            ((-60, 60), -0.10697486f32),
            ((-60, 70), -0.10814563f32),
            ((-60, 80), -0.10756317f32),
            ((-60, 90), -0.10590332f32),
            ((-60, 100), -0.10525697f32),
            ((-50, -100), -0.08205913f32),
            ((-50, -90), -0.08393769f32),
            ((-50, -80), -0.086751714f32),
            ((-50, -70), -0.089658506f32),
            ((-50, -60), -0.092683166f32),
            ((-50, -50), -0.09480399f32),
            ((-50, -40), -0.09606384f32),
            ((-50, -30), -0.09774114f32),
            ((-50, -20), -0.100446284f32),
            ((-50, -10), -0.10084822f32),
            ((-50, 0), -0.10314222f32),
            ((-50, 10), -0.10293749f32),
            ((-50, 20), -0.10586517f32),
            ((-50, 30), -0.10697569f32),
            ((-50, 40), -0.10719836f32),
            ((-50, 50), -0.10689005f32),
            ((-50, 60), -0.10781257f32),
            ((-50, 70), -0.10913751f32),
            ((-50, 80), -0.111755826f32),
            ((-50, 90), -0.10983417f32),
            ((-50, 100), -0.10947363f32),
            ((-40, -100), -0.0824882f32),
            ((-40, -90), -0.08360703f32),
            ((-40, -80), -0.086381115f32),
            ((-40, -70), -0.09064654f32),
            ((-40, -60), -0.093374886f32),
            ((-40, -50), -0.09642702f32),
            ((-40, -40), -0.09817896f32),
            ((-40, -30), -0.100624494f32),
            ((-40, -20), -0.10402103f32),
            ((-40, -10), -0.103324674f32),
            ((-40, 0), -0.10734479f32),
            ((-40, 10), -0.10787663f32),
            ((-40, 20), -0.11011673f32),
            ((-40, 30), -0.10879173f32),
            ((-40, 40), -0.10899488f32),
            ((-40, 50), -0.1093022f32),
            ((-40, 60), -0.11100974f32),
            ((-40, 70), -0.11430037f32),
            ((-40, 80), -0.11370994f32),
            ((-40, 90), -0.1117007f32),
            ((-40, 100), -0.111225165f32),
            ((-30, -100), -0.08073887f32),
            ((-30, -90), -0.08562371f32),
            ((-30, -80), -0.08811793f32),
            ((-30, -70), -0.092022136f32),
            ((-30, -60), -0.09238706f32),
            ((-30, -50), -0.09777247f32),
            ((-30, -40), -0.10047619f32),
            ((-30, -30), -0.1017582f32),
            ((-30, -20), -0.10325858f32),
            ((-30, -10), -0.10587716f32),
            ((-30, 0), -0.10807945f32),
            ((-30, 10), -0.11133594f32),
            ((-30, 20), -0.11149085f32),
            ((-30, 30), -0.11169845f32),
            ((-30, 40), -0.11191458f32),
            ((-30, 50), -0.11245423f32),
            ((-30, 60), -0.11395778f32),
            ((-30, 70), -0.116144754f32),
            ((-30, 80), -0.115703195f32),
            ((-30, 90), -0.11417798f32),
            ((-30, 100), -0.113027185f32),
            ((-20, -100), -0.081831336f32),
            ((-20, -90), -0.08537665f32),
            ((-20, -80), -0.08823441f32),
            ((-20, -70), -0.09004638f32),
            ((-20, -60), -0.091682106f32),
            ((-20, -50), -0.09560484f32),
            ((-20, -40), -0.100073636f32),
            ((-20, -30), -0.102195345f32),
            ((-20, -20), -0.103985146f32),
            ((-20, -10), -0.105535336f32),
            ((-20, 0), -0.11013269f32),
            ((-20, 10), -0.1115511f32),
            ((-20, 20), -0.113062836f32),
            ((-20, 30), -0.112847894f32),
            ((-20, 40), -0.11428483f32),
            ((-20, 50), -0.11667834f32),
            ((-20, 60), -0.117276795f32),
            ((-20, 70), -0.11689238f32),
            ((-20, 80), -0.1160782f32),
            ((-20, 90), -0.11455002f32),
            ((-20, 100), -0.11329481f32),
            ((-10, -100), -0.08257191f32),
            ((-10, -90), -0.085004464f32),
            ((-10, -80), -0.08695547f32),
            ((-10, -70), -0.08808699f32),
            ((-10, -60), -0.091026604f32),
            ((-10, -50), -0.09544293f32),
            ((-10, -40), -0.09790615f32),
            ((-10, -30), -0.10184401f32),
            ((-10, -20), -0.10396968f32),
            ((-10, -10), -0.1076317f32),
            ((-10, 0), -0.11115202f32),
            ((-10, 10), -0.11212789f32),
            ((-10, 20), -0.11348673f32),
            ((-10, 30), -0.11471321f32),
            ((-10, 40), -0.114718616f32),
            ((-10, 50), -0.11725365f32),
            ((-10, 60), -0.11695717f32),
            ((-10, 70), -0.116616994f32),
            ((-10, 80), -0.11536371f32),
            ((-10, 90), -0.11446484f32),
            ((-10, 100), -0.11369774f32),
            ((0, -100), -0.08459158f32),
            ((0, -90), -0.08656791f32),
            ((0, -80), -0.08754034f32),
            ((0, -70), -0.08878641f32),
            ((0, -60), -0.09282567f32),
            ((0, -50), -0.09490024f32),
            ((0, -40), -0.09933582f32),
            ((0, -30), -0.10083613f32),
            ((0, -20), -0.104990706f32),
            ((0, -10), -0.10789699f32),
            ((0, 0), -0.10978986f32),
            ((0, 10), -0.11214093f32),
            ((0, 20), -0.11311828f32),
            ((0, 30), -0.11421281f32),
            ((0, 40), -0.11489451f32),
            ((0, 50), -0.11654575f32),
            ((0, 60), -0.116693474f32),
            ((0, 70), -0.11676903f32),
            ((0, 80), -0.11486762f32),
            ((0, 90), -0.11349271f32),
            ((0, 100), -0.11301751f32),
            ((10, -100), -0.08532186f32),
            ((10, -90), -0.087390676f32),
            ((10, -80), -0.08947607f32),
            ((10, -70), -0.091350235f32),
            ((10, -60), -0.092257306f32),
            ((10, -50), -0.094102554f32),
            ((10, -40), -0.09877145f32),
            ((10, -30), -0.101968475f32),
            ((10, -20), -0.10476847f32),
            ((10, -10), -0.10820749f32),
            ((10, 0), -0.1099116f32),
            ((10, 10), -0.1106353f32),
            ((10, 20), -0.11173092f32),
            ((10, 30), -0.11349803f32),
            ((10, 40), -0.11299823f32),
            ((10, 50), -0.11539698f32),
            ((10, 60), -0.11715141f32),
            ((10, 70), -0.11558097f32),
            ((10, 80), -0.114774175f32),
            ((10, 90), -0.11429333f32),
            ((10, 100), -0.112418234f32),
            ((20, -100), -0.08536324f32),
            ((20, -90), -0.08702316f32),
            ((20, -80), -0.09097032f32),
            ((20, -70), -0.09171111f32),
            ((20, -60), -0.092209786f32),
            ((20, -50), -0.09390856f32),
            ((20, -40), -0.09674665f32),
            ((20, -30), -0.0995738f32),
            ((20, -20), -0.10170178f32),
            ((20, -10), -0.107144f32),
            ((20, 0), -0.10934077f32),
            ((20, 10), -0.11114335f32),
            ((20, 20), -0.11120629f32),
            ((20, 30), -0.11104426f32),
            ((20, 40), -0.109872885f32),
            ((20, 50), -0.11435944f32),
            ((20, 60), -0.11630697f32),
            ((20, 70), -0.11403515f32),
            ((20, 80), -0.11370773f32),
            ((20, 90), -0.11165723f32),
            ((20, 100), -0.11122058f32),
            ((30, -100), -0.083562575f32),
            ((30, -90), -0.08608784f32),
            ((30, -80), -0.08794823f32),
            ((30, -70), -0.091189116f32),
            ((30, -60), -0.093153656f32),
            ((30, -50), -0.0951614f32),
            ((30, -40), -0.0953993f32),
            ((30, -30), -0.096983574f32),
            ((30, -20), -0.10012217f32),
            ((30, -10), -0.10612148f32),
            ((30, 0), -0.10943102f32),
            ((30, 10), -0.10920269f32),
            ((30, 20), -0.10791202f32),
            ((30, 30), -0.10835938f32),
            ((30, 40), -0.10786465f32),
            ((30, 50), -0.11130254f32),
            ((30, 60), -0.1126149f32),
            ((30, 70), -0.110773936f32),
            ((30, 80), -0.109979525f32),
            ((30, 90), -0.10987309f32),
            ((30, 100), -0.108747214f32),
            ((40, -100), -0.08093807f32),
            ((40, -90), -0.08576739f32),
            ((40, -80), -0.085233085f32),
            ((40, -70), -0.09012596f32),
            ((40, -60), -0.09217769f32),
            ((40, -50), -0.09351354f32),
            ((40, -40), -0.09589194f32),
            ((40, -30), -0.09674299f32),
            ((40, -20), -0.09954912f32),
            ((40, -10), -0.10170849f32),
            ((40, 0), -0.10476398f32),
            ((40, 10), -0.10546719f32),
            ((40, 20), -0.10537742f32),
            ((40, 30), -0.105657674f32),
            ((40, 40), -0.10615531f32),
            ((40, 50), -0.10808634f32),
            ((40, 60), -0.10634413f32),
            ((40, 70), -0.107364416f32),
            ((40, 80), -0.10856771f32),
            ((40, 90), -0.10862812f32),
            ((40, 100), -0.10680454f32),
            ((50, -100), -0.082533084f32),
            ((50, -90), -0.086810954f32),
            ((50, -80), -0.085271746f32),
            ((50, -70), -0.08944471f32),
            ((50, -60), -0.09193231f32),
            ((50, -50), -0.09362271f32),
            ((50, -40), -0.09453575f32),
            ((50, -30), -0.095691115f32),
            ((50, -20), -0.098154165f32),
            ((50, -10), -0.097271696f32),
            ((50, 0), -0.09942495f32),
            ((50, 10), -0.10164021f32),
            ((50, 20), -0.10168414f32),
            ((50, 30), -0.104937024f32),
            ((50, 40), -0.10539722f32),
            ((50, 50), -0.10424481f32),
            ((50, 60), -0.10219841f32),
            ((50, 70), -0.10340266f32),
            ((50, 80), -0.106310576f32),
            ((50, 90), -0.10595322f32),
            ((50, 100), -0.10645929f32),
            ((60, -100), -0.08115639f32),
            ((60, -90), -0.08455347f32),
            ((60, -80), -0.08534711f32),
            ((60, -70), -0.08780121f32),
            ((60, -60), -0.090132974f32),
            ((60, -50), -0.091330595f32),
            ((60, -40), -0.091192245f32),
            ((60, -30), -0.0924395f32),
            ((60, -20), -0.09527585f32),
            ((60, -10), -0.09565725f32),
            ((60, 0), -0.09630064f32),
            ((60, 10), -0.09673829f32),
            ((60, 20), -0.09658762f32),
            ((60, 30), -0.09961178f32),
            ((60, 40), -0.100632355f32),
            ((60, 50), -0.10012698f32),
            ((60, 60), -0.09957688f32),
            ((60, 70), -0.10111454f32),
            ((60, 80), -0.10357678f32),
            ((60, 90), -0.104492605f32),
            ((60, 100), -0.10421045f32),
            ((70, -100), -0.074997574f32),
            ((70, -90), -0.07923891f32),
            ((70, -80), -0.08061024f32),
            ((70, -70), -0.08389824f32),
            ((70, -60), -0.08752339f32),
            ((70, -50), -0.090026386f32),
            ((70, -40), -0.09045799f32),
            ((70, -30), -0.091568656f32),
            ((70, -20), -0.09186759f32),
            ((70, -10), -0.09279721f32),
            ((70, 0), -0.093260504f32),
            ((70, 10), -0.092824616f32),
            ((70, 20), -0.093960315f32),
            ((70, 30), -0.09660603f32),
            ((70, 40), -0.09790762f32),
            ((70, 50), -0.09831002f32),
            ((70, 60), -0.09862129f32),
            ((70, 70), -0.1009444f32),
            ((70, 80), -0.101910815f32),
            ((70, 90), -0.10201355f32),
            ((70, 100), -0.1022472f32),
            ((80, -100), -0.07063179f32),
            ((80, -90), -0.073081866f32),
            ((80, -80), -0.0773198f32),
            ((80, -70), -0.079035714f32),
            ((80, -60), -0.08466645f32),
            ((80, -50), -0.08781248f32),
            ((80, -40), -0.087890774f32),
            ((80, -30), -0.09016053f32),
            ((80, -20), -0.09094465f32),
            ((80, -10), -0.09138842f32),
            ((80, 0), -0.090936236f32),
            ((80, 10), -0.09181613f32),
            ((80, 20), -0.09273602f32),
            ((80, 30), -0.09400447f32),
            ((80, 40), -0.094502196f32),
            ((80, 50), -0.09436651f32),
            ((80, 60), -0.09598513f32),
            ((80, 70), -0.098289296f32),
            ((80, 80), -0.10086883f32),
            ((80, 90), -0.101560704f32),
            ((80, 100), -0.10193728f32),
            ((90, -100), -0.07009827f32),
            ((90, -90), -0.071226865f32),
            ((90, -80), -0.07469955f32),
            ((90, -70), -0.07523824f32),
            ((90, -60), -0.07865613f32),
            ((90, -50), -0.084405504f32),
            ((90, -40), -0.085147366f32),
            ((90, -30), -0.08834492f32),
            ((90, -20), -0.08923916f32),
            ((90, -10), -0.08832547f32),
            ((90, 0), -0.087817885f32),
            ((90, 10), -0.09013721f32),
            ((90, 20), -0.091518745f32),
            ((90, 30), -0.091617286f32),
            ((90, 40), -0.0920376f32),
            ((90, 50), -0.09236775f32),
            ((90, 60), -0.094668776f32),
            ((90, 70), -0.096736684f32),
            ((90, 80), -0.099345334f32),
            ((90, 90), -0.10124628f32),
            ((90, 100), -0.10255872f32),
            ((100, -100), -0.06807774f32),
            ((100, -90), -0.07107438f32),
            ((100, -80), -0.072487816f32),
            ((100, -70), -0.075734794f32),
            ((100, -60), -0.07796392f32),
            ((100, -50), -0.08121155f32),
            ((100, -40), -0.08184202f32),
            ((100, -30), -0.08378057f32),
            ((100, -20), -0.0849825f32),
            ((100, -10), -0.08561178f32),
            ((100, 0), -0.08618045f32),
            ((100, 10), -0.08772436f32),
            ((100, 20), -0.08878901f32),
            ((100, 30), -0.08852096f32),
            ((100, 40), -0.0906315f32),
            ((100, 50), -0.091744505f32),
            ((100, 60), -0.093411185f32),
            ((100, 70), -0.09586958f32),
            ((100, 80), -0.098537765f32),
            ((100, 90), -0.10159851f32),
            ((100, 100), -0.10332601f32),
        ];

        for ((x, z), result) in values {
            let pos = &NoisePos::Unblended(UnblendedNoisePos::new(x, 60, z));
            assert_eq!(func.sample_mut(pos, &FakeEnvironment {}), result as f64);
        }
    }
}
