use std::rc::Rc;

use noise::{InternalNoise, NoiseFunction, ShiftedNoiseFunction};
use offset::{ShiftAFunction, ShiftBFunction};
use unary::{ClampFunction, UnaryFunction, UnaryType};

use crate::world_gen::blender::Blender;

mod noise;
mod offset;
mod unary;

#[derive(Clone)]
pub enum DensityFunction<'a> {
    Clamp(ClampFunction<'a>),
    Unary(UnaryFunction<'a>),
    Noise(NoiseFunction<'a>),
    ShiftA(ShiftAFunction<'a>),
    ShiftB(ShiftBFunction<'a>),
    ShiftedNoise(ShiftedNoiseFunction<'a>),
}

impl<'a> DensityFunction<'a> {
    #[inline]
    pub fn sample(&self, pos: &impl NoisePos) -> f64 {
        match self {
            Self::Clamp(func) => func.sample(pos),
            Self::Unary(func) => func.sample(pos),
            Self::Noise(func) => func.sample(pos),
            Self::ShiftA(func) => func.sample(pos),
            Self::ShiftB(func) => func.sample(pos),
            Self::ShiftedNoise(func) => func.sample(pos),
        }
    }

    #[inline]
    pub fn apply(&'a self, visitor: &'a impl Visitor) -> DensityFunction<'a> {
        match self {
            Self::Clamp(func) => func.apply(visitor),
            Self::Unary(func) => func.apply(visitor),
            Self::Noise(func) => func.apply(visitor),
            Self::ShiftA(func) => func.apply(visitor),
            Self::ShiftB(func) => func.apply(visitor),
            Self::ShiftedNoise(func) => func.apply(visitor),
        }
    }

    #[inline]
    pub fn fill(&self, densities: &[f64], applier: &impl Applier) -> Vec<f64> {
        match self {
            Self::Clamp(func) => func.fill(densities, applier),
            Self::Unary(func) => func.fill(densities, applier),
            Self::Noise(func) => func.fill(densities, applier),
            Self::ShiftA(func) => func.fill(densities, applier),
            Self::ShiftB(func) => func.fill(densities, applier),
            Self::ShiftedNoise(func) => func.fill(densities, applier),
        }
    }

    #[inline]
    pub fn max(&self) -> f64 {
        match self {
            Self::Clamp(func) => func.max(),
            Self::Unary(func) => func.max(),
            Self::Noise(func) => func.max(),
            Self::ShiftA(func) => func.max(),
            Self::ShiftB(func) => func.max(),
            Self::ShiftedNoise(func) => func.max(),
        }
    }

    #[inline]
    pub fn min(&self) -> f64 {
        match self {
            Self::Clamp(func) => func.min(),
            Self::Unary(func) => func.min(),
            Self::Noise(func) => func.min(),
            Self::ShiftA(func) => func.min(),
            Self::ShiftB(func) => func.min(),
            Self::ShiftedNoise(func) => func.min(),
        }
    }

    pub fn clamp(&self, max: f64, min: f64) -> Self {
        Self::Clamp(ClampFunction {
            input: Rc::new(self.clone()),
            min,
            max,
        })
    }

    pub fn abs(&self) -> Self {
        Self::Unary(UnaryFunction::create(UnaryType::Abs, Rc::new(self.clone())))
    }

    pub fn square(&self) -> Self {
        Self::Unary(UnaryFunction::create(
            UnaryType::Square,
            Rc::new(self.clone()),
        ))
    }

    pub fn cube(&self) -> Self {
        Self::Unary(UnaryFunction::create(
            UnaryType::Cube,
            Rc::new(self.clone()),
        ))
    }

    pub fn half_negative(&self) -> Self {
        Self::Unary(UnaryFunction::create(
            UnaryType::HalfNeg,
            Rc::new(self.clone()),
        ))
    }

    pub fn quarter_negative(&self) -> Self {
        Self::Unary(UnaryFunction::create(
            UnaryType::QuartNeg,
            Rc::new(self.clone()),
        ))
    }

    pub fn squeeze(&self) -> Self {
        Self::Unary(UnaryFunction::create(
            UnaryType::Squeeze,
            Rc::new(self.clone()),
        ))
    }
}

pub trait NoisePos {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn z(&self) -> i32;

    fn get_blender(&self) -> &Blender {
        &Blender {}
    }
}

pub trait Applier {
    fn at(&self, index: i32) -> impl NoisePos;

    fn fill<'a>(&self, densities: &[f64], function: &impl DensityFunctionImpl<'a>) -> Vec<f64>;
}

pub trait Visitor {
    fn apply(&self, function: &DensityFunction) -> DensityFunction;

    fn apply_internal_noise<'a>(&self, function: Rc<InternalNoise<'a>>) -> Rc<InternalNoise<'a>> {
        function.clone()
    }
}

pub trait DensityFunctionImpl<'a> {
    fn sample(&self, pos: &impl NoisePos) -> f64;

    fn fill(&self, densities: &[f64], applier: &impl Applier) -> Vec<f64>;

    fn apply(&'a self, visitor: &'a impl Visitor) -> DensityFunction<'a>;

    fn min(&self) -> f64;

    fn max(&self) -> f64;
}

pub trait UnaryDensityFunction<'a>: DensityFunctionImpl<'a> {
    fn input(&self) -> &DensityFunction;

    fn apply_density(&self, density: f64) -> f64;
}

pub trait OffsetDensityFunction<'a>: DensityFunctionImpl<'a> {
    fn offset_noise(&self) -> &InternalNoise<'a>;

    fn sample_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        self.offset_noise()
            .sample(x * 0.25f64, y * 0.25f64, z * 0.25f64)
            * 4f64
    }
}
