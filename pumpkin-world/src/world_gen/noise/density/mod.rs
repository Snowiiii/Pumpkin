use std::sync::Arc;

use blend::{BlendAlphaFunction, BlendDensityFunction, BlendOffsetFunction};
use math::{BinaryFunction, BinaryType, LinearFunction};
use noise::{InternalNoise, InterpolatedNoiseSampler, NoiseFunction, ShiftedNoiseFunction};
use offset::{ShiftAFunction, ShiftBFunction};
use spline::SplineFunction;
use unary::{ClampFunction, UnaryFunction, UnaryType};

use crate::world_gen::blender::Blender;

use super::clamped_map;

mod blend;
mod math;
mod noise;
mod offset;
pub mod spline;
mod unary;

pub mod built_in_noises {
    use std::sync::{Arc, LazyLock};

    use crate::world_gen::{
        chunk::{MAX_COLUMN_HEIGHT, MIN_HEIGHT},
        implementation::overworld::terrain_params::{
            create_factor_spline, create_jaggedness_spline, create_offset_spline,
        },
        noise::builtin_noise_params,
    };

    use super::{
        blend::{BlendAlphaFunction, BlendOffsetFunction},
        lerp_density,
        noise::{InternalNoise, InterpolatedNoiseSampler, NoiseFunction, ShiftedNoiseFunction},
        offset::{ShiftAFunction, ShiftBFunction},
        spline::SplineFunction,
        ConstantFunction, DensityFunction, YClampedFunction,
    };

    pub struct SlopedCheeseResult<'a> {
        offset: DensityFunction<'a>,
        factor: DensityFunction<'a>,
        depth: DensityFunction<'a>,
        jaggedness: DensityFunction<'a>,
        sloped_cheese: DensityFunction<'a>,
    }

    type BuiltInNoise = LazyLock<DensityFunction<'static>>;
    type BuiltInSlopedCheese = LazyLock<SlopedCheeseResult<'static>>;

    pub static ZERO: BuiltInNoise =
        LazyLock::new(|| DensityFunction::Constant(ConstantFunction::new(0f64)));

    pub static TEN: BuiltInNoise =
        LazyLock::new(|| DensityFunction::Constant(ConstantFunction::new(10f64)));

    pub static Y: BuiltInNoise = LazyLock::new(|| {
        DensityFunction::ClampedY(YClampedFunction {
            from: MIN_HEIGHT * 2,
            to: MAX_COLUMN_HEIGHT * 2,
            from_val: (MIN_HEIGHT * 2) as f64,
            to_val: (MAX_COLUMN_HEIGHT * 2) as f64,
        })
    });

    pub static SHIFT_X: BuiltInNoise = LazyLock::new(|| {
        DensityFunction::ShiftA(ShiftAFunction::new(Arc::new(InternalNoise::new(
            builtin_noise_params::OFFSET.clone(),
            None,
        ))))
    });

    pub static SHIFT_Z: BuiltInNoise = LazyLock::new(|| {
        DensityFunction::ShiftB(ShiftBFunction::new(Arc::new(InternalNoise::new(
            builtin_noise_params::OFFSET.clone(),
            None,
        ))))
    });

    pub static BASE_3D_NOISE_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        DensityFunction::InterpolatedNoise(InterpolatedNoiseSampler::create_base_3d_noise_function(
            0.25f64, 0.125f64, 80f64, 160f64, 8f64,
        ))
    });

    pub static BASE_3D_NOISE_NETHER: BuiltInNoise = LazyLock::new(|| {
        DensityFunction::InterpolatedNoise(InterpolatedNoiseSampler::create_base_3d_noise_function(
            0.25f64, 0.375f64, 80f64, 60f64, 8f64,
        ))
    });

    pub static BASE_3D_NOISE_END: BuiltInNoise = LazyLock::new(|| {
        DensityFunction::InterpolatedNoise(InterpolatedNoiseSampler::create_base_3d_noise_function(
            0.25f64, 0.25f64, 80f64, 160f64, 4f64,
        ))
    });

    pub static CONTINENTS_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
            Arc::new(SHIFT_X.clone()),
            Arc::new(ZERO.clone()),
            Arc::new(SHIFT_Z.clone()),
            0.25f64,
            0f64,
            Arc::new(InternalNoise::new(
                builtin_noise_params::CONTINENTALNESS.clone(),
                None,
            )),
        ))
    });

    pub static EROSION_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
            Arc::new(SHIFT_X.clone()),
            Arc::new(ZERO.clone()),
            Arc::new(SHIFT_Z.clone()),
            0.25f64,
            0f64,
            Arc::new(InternalNoise::new(
                builtin_noise_params::EROSION.clone(),
                None,
            )),
        ))
    });

    pub static RIDGES_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
            Arc::new(SHIFT_X.clone()),
            Arc::new(ZERO.clone()),
            Arc::new(SHIFT_Z.clone()),
            0.25f64,
            0f64,
            Arc::new(InternalNoise::new(
                builtin_noise_params::RIDGE.clone(),
                None,
            )),
        ))
    });

    pub static RIDGES_FOLDED_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        RIDGES_OVERWORLD
            .abs()
            .add_const(-0.6666666666666666f64)
            .abs()
            .add_const(-0.3333333333333333f64)
            .mul_const(-3f64)
    });

    pub static OVERWORLD_SLOPED_CHEESE: BuiltInSlopedCheese = LazyLock::new(|| {
        sloped_cheese_function(
            DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    builtin_noise_params::JAGGED.clone(),
                    None,
                )),
                1500f64,
                0f64,
            )),
            CONTINENTS_OVERWORLD.clone(),
            EROSION_OVERWORLD.clone(),
            RIDGES_OVERWORLD.clone(),
            RIDGES_FOLDED_OVERWORLD.clone(),
            false,
        )
    });

    fn sloped_cheese_function<'a>(
        jagged_noise: DensityFunction<'a>,
        continents: DensityFunction<'a>,
        erosion: DensityFunction<'a>,
        ridges: DensityFunction<'a>,
        ridges_folded: DensityFunction<'a>,
        amplified: bool,
    ) -> SlopedCheeseResult<'a> {
        let offset = lerp_density(
            DensityFunction::BlendAlpha(BlendAlphaFunction {}),
            DensityFunction::Spline(SplineFunction::new(Arc::new(create_offset_spline(
                continents.clone(),
                erosion.clone(),
                ridges.clone(),
                amplified,
            ))))
            .add_const(-0.50375f32 as f64),
            DensityFunction::BlendOffset(BlendOffsetFunction {}),
        );

        let factor = lerp_density(
            DensityFunction::BlendAlpha(BlendAlphaFunction {}),
            DensityFunction::Spline(SplineFunction::new(Arc::new(create_factor_spline(
                continents.clone(),
                erosion.clone(),
                ridges.clone(),
                ridges_folded.clone(),
                amplified,
            )))),
            TEN.clone(),
        );

        let depth = DensityFunction::ClampedY(YClampedFunction {
            from: -64,
            to: 320,
            from_val: 1.564,
            to_val: -1.5f64,
        })
        .add(offset.clone());

        let jaggedness = lerp_density(
            DensityFunction::BlendAlpha(BlendAlphaFunction {}),
            DensityFunction::Spline(SplineFunction::new(Arc::new(create_jaggedness_spline(
                continents,
                erosion,
                ridges,
                ridges_folded,
                amplified,
            )))),
            ZERO.clone(),
        );

        let density1 = jaggedness.mul(jagged_noise.half_negative());
        let density2 = DensityFunction::Constant(ConstantFunction::new(4f64))
            .mul(depth.add(density1).mul(factor.clone()).quarter_negative());

        let sloped_cheese = density2.add(BASE_3D_NOISE_OVERWORLD.clone());

        SlopedCheeseResult {
            offset,
            factor,
            depth,
            jaggedness,
            sloped_cheese,
        }
    }
}

pub fn peaks_valleys_noise(variance: f32) -> f32 {
    -((variance.abs() - 0.6666667f32).abs() - 0.33333334f32) * 3f32
}

#[derive(Clone)]
pub enum DensityFunction<'a> {
    Clamp(ClampFunction<'a>),
    Unary(UnaryFunction<'a>),
    Noise(NoiseFunction<'a>),
    ShiftA(ShiftAFunction<'a>),
    ShiftB(ShiftBFunction<'a>),
    ShiftedNoise(ShiftedNoiseFunction<'a>),
    Spline(SplineFunction<'a>),
    Constant(ConstantFunction),
    Linear(LinearFunction<'a>),
    Binary(BinaryFunction<'a>),
    BlendOffset(BlendOffsetFunction),
    BlendAlpha(BlendAlphaFunction),
    BlendDensity(BlendDensityFunction<'a>),
    ClampedY(YClampedFunction),
    InterpolatedNoise(InterpolatedNoiseSampler),
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
            Self::Spline(func) => func.sample(pos),
            Self::Constant(func) => func.sample(pos),
            Self::Linear(func) => func.sample(pos),
            Self::Binary(func) => func.sample(pos),
            Self::BlendOffset(func) => func.sample(pos),
            Self::BlendAlpha(func) => func.sample(pos),
            Self::BlendDensity(func) => func.sample(pos),
            Self::ClampedY(func) => func.sample(pos),
            Self::InterpolatedNoise(func) => func.sample(pos),
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
            Self::Spline(func) => func.apply(visitor),
            Self::Constant(func) => func.apply(visitor),
            Self::Linear(func) => func.apply(visitor),
            Self::Binary(func) => func.apply(visitor),
            Self::BlendOffset(func) => func.apply(visitor),
            Self::BlendAlpha(func) => func.apply(visitor),
            Self::BlendDensity(func) => func.apply(visitor),
            Self::ClampedY(func) => func.apply(visitor),
            Self::InterpolatedNoise(func) => func.apply(visitor),
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
            Self::Spline(func) => func.fill(densities, applier),
            Self::Constant(func) => func.fill(densities, applier),
            Self::Linear(func) => func.fill(densities, applier),
            Self::Binary(func) => func.fill(densities, applier),
            Self::BlendOffset(func) => func.fill(densities, applier),
            Self::BlendAlpha(func) => func.fill(densities, applier),
            Self::BlendDensity(func) => func.fill(densities, applier),
            Self::ClampedY(func) => func.fill(densities, applier),
            Self::InterpolatedNoise(func) => func.fill(densities, applier),
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
            Self::Spline(func) => func.max(),
            Self::Constant(func) => func.max(),
            Self::Linear(func) => func.max(),
            Self::Binary(func) => func.max(),
            Self::BlendOffset(func) => func.max(),
            Self::BlendAlpha(func) => func.max(),
            Self::BlendDensity(func) => func.max(),
            Self::ClampedY(func) => func.max(),
            Self::InterpolatedNoise(func) => func.max(),
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
            Self::Spline(func) => func.min(),
            Self::Constant(func) => func.min(),
            Self::Linear(func) => func.min(),
            Self::Binary(func) => func.min(),
            Self::BlendOffset(func) => func.min(),
            Self::BlendAlpha(func) => func.min(),
            Self::BlendDensity(func) => func.min(),
            Self::ClampedY(func) => func.min(),
            Self::InterpolatedNoise(func) => func.min(),
        }
    }

    pub fn clamp(&self, max: f64, min: f64) -> Self {
        Self::Clamp(ClampFunction {
            input: Arc::new(self.clone()),
            min,
            max,
        })
    }

    pub fn abs(&self) -> Self {
        Self::Unary(UnaryFunction::create(
            UnaryType::Abs,
            Arc::new(self.clone()),
        ))
    }

    pub fn square(&self) -> Self {
        Self::Unary(UnaryFunction::create(
            UnaryType::Square,
            Arc::new(self.clone()),
        ))
    }

    pub fn cube(&self) -> Self {
        Self::Unary(UnaryFunction::create(
            UnaryType::Cube,
            Arc::new(self.clone()),
        ))
    }

    pub fn half_negative(&self) -> Self {
        Self::Unary(UnaryFunction::create(
            UnaryType::HalfNeg,
            Arc::new(self.clone()),
        ))
    }

    pub fn quarter_negative(&self) -> Self {
        Self::Unary(UnaryFunction::create(
            UnaryType::QuartNeg,
            Arc::new(self.clone()),
        ))
    }

    pub fn squeeze(&self) -> Self {
        Self::Unary(UnaryFunction::create(
            UnaryType::Squeeze,
            Arc::new(self.clone()),
        ))
    }

    pub fn add_const(&self, val: f64) -> Self {
        self.add(Self::Constant(ConstantFunction::new(val)))
    }

    pub fn add(&self, other: DensityFunction<'a>) -> Self {
        BinaryFunction::create(BinaryType::Add, self.clone(), other)
    }

    pub fn mul_const(&self, val: f64) -> Self {
        self.mul(Self::Constant(ConstantFunction::new(val)))
    }

    pub fn mul(&self, other: DensityFunction<'a>) -> Self {
        BinaryFunction::create(BinaryType::Mul, self.clone(), other)
    }

    pub fn binary_min(&self, other: DensityFunction<'a>) -> Self {
        BinaryFunction::create(BinaryType::Min, self.clone(), other)
    }

    pub fn binary_max(&self, other: DensityFunction<'a>) -> Self {
        BinaryFunction::create(BinaryType::Max, self.clone(), other)
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

    fn apply_internal_noise<'a>(&self, function: Arc<InternalNoise<'a>>) -> Arc<InternalNoise<'a>> {
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

#[derive(Clone)]
pub struct ConstantFunction {
    value: f64,
}

impl ConstantFunction {
    pub fn new(value: f64) -> Self {
        ConstantFunction { value }
    }
}

impl<'a> DensityFunctionImpl<'a> for ConstantFunction {
    fn sample(&self, _pos: &impl NoisePos) -> f64 {
        self.value
    }

    fn fill(&self, densities: &[f64], _applier: &impl Applier) -> Vec<f64> {
        densities.iter().map(|_| self.value).collect()
    }

    fn apply(&'a self, visitor: &'a impl Visitor) -> DensityFunction<'a> {
        visitor.apply(&DensityFunction::Constant(self.clone()))
    }

    fn min(&self) -> f64 {
        self.value
    }

    fn max(&self) -> f64 {
        self.value
    }
}

#[derive(Clone)]
pub struct YClampedFunction {
    from: i32,
    to: i32,
    from_val: f64,
    to_val: f64,
}

impl<'a> DensityFunctionImpl<'a> for YClampedFunction {
    fn sample(&self, pos: &impl NoisePos) -> f64 {
        clamped_map(
            pos.y() as f64,
            self.from as f64,
            self.to as f64,
            self.from_val,
            self.to_val,
        )
    }

    fn min(&self) -> f64 {
        self.from_val.min(self.to_val)
    }

    fn max(&self) -> f64 {
        self.from_val.max(self.to_val)
    }

    fn fill(&self, densities: &[f64], applier: &impl Applier) -> Vec<f64> {
        applier.fill(densities, self)
    }

    fn apply(&'a self, visitor: &'a impl Visitor) -> DensityFunction<'a> {
        visitor.apply(&DensityFunction::ClampedY(self.clone()))
    }
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

pub fn lerp_density<'a>(
    delta: DensityFunction<'a>,
    start: DensityFunction<'a>,
    end: DensityFunction<'a>,
) -> DensityFunction<'a> {
    let func_2 = delta.mul_const(-1f64).add_const(1f64);
    start.mul(func_2.clone()).add(end.mul(func_2))
}
