use std::sync::Arc;

use basic::{ConstantFunction, RangeFunction, WrapperFunction, WrapperType};
use blend::BlendDensityFunction;
use built_in_density_function::BLEND_ALPHA;
use component_functions::{
    ComponentReference, ComponentReferenceMap, ComponentReferenceMath, ConverterEnvironment,
    NoEnvironment, SharedComponentReference,
};
use enum_dispatch::enum_dispatch;
use noise::{InternalNoise, NoiseFunction};

use crate::world_gen::{blender::Blender, chunk_noise::ChunkNoisePos};

use super::perlin::DoublePerlinNoiseParameters;

pub mod basic;
mod blend;
pub mod component_functions;
pub mod end;
pub mod math;
pub mod noise;
mod offset;
mod spline;
mod terrain_helpers;
mod unary;
mod weird;

#[derive(Debug)]
#[enum_dispatch(NoisePosImpl)]
pub enum NoisePos {
    Unblended(UnblendedNoisePos),
    Chunk(ChunkNoisePos),
}

#[derive(Debug)]
pub struct UnblendedNoisePos {
    x: i32,
    y: i32,
    z: i32,
}

impl UnblendedNoisePos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl NoisePosImpl for UnblendedNoisePos {
    fn x(&self) -> i32 {
        self.x
    }

    fn y(&self) -> i32 {
        self.y
    }

    fn z(&self) -> i32 {
        self.z
    }

    fn get_blender(&self) -> Blender {
        Blender::NO_BLEND
    }
}

#[enum_dispatch]
pub trait NoisePosImpl {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn z(&self) -> i32;

    fn get_blender(&self) -> Blender;
}

pub mod built_in_density_function {
    use std::sync::Arc;

    use lazy_static::lazy_static;

    use crate::world_gen::noise::built_in_noise_params;
    use crate::world_gen::positions::{MAX_COLUMN_HEIGHT, MIN_HEIGHT};

    use super::{apply_blending, noise_in_range, vertical_range_choice};

    use super::basic::{
        ConstantFunction, RangeFunction, WrapperFunction, WrapperType, YClampedFunction,
    };
    use super::blend::{BlendAlphaFunction, BlendOffsetFunction};
    use super::component_functions::{
        ComponentReferenceMap, ComponentReferenceMath, NoEnvironment, SharedComponentReference,
    };
    use super::end::EndIslandFunction;
    use super::noise::{
        InternalNoise, InterpolatedNoiseFunction, NoiseFunction, ShiftedNoiseFunction,
    };
    use super::offset::{ShiftAFunction, ShiftBFunction};

    use super::spline::{ImmutableSplineRef, SplineFunction};

    use super::terrain_helpers::{
        create_factor_spline, create_jaggedness_spline, create_offset_spline,
    };

    use super::weird::{RarityMapper, WierdScaledFunction};

    lazy_static! {
        pub static ref ZERO: SharedComponentReference = ConstantFunction::new(0f64).into();
        pub static ref TEN: SharedComponentReference = ConstantFunction::new(10f64).into();
        pub static ref BLEND_ALPHA: SharedComponentReference = BlendAlphaFunction::INSTANCE.into();
        pub static ref BLEND_OFFSET: SharedComponentReference =
            BlendOffsetFunction::INSTANCE.into();
        pub static ref Y: SharedComponentReference = YClampedFunction::new(
            MIN_HEIGHT * 2,
            MAX_COLUMN_HEIGHT as i32 * 2,
            (MIN_HEIGHT * 2) as f64,
            ((MAX_COLUMN_HEIGHT as i32) * 2) as f64
        )
        .into();
        pub static ref SHIFT_X: SharedComponentReference =
            WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                    ShiftAFunction::new(Arc::new(InternalNoise::new(
                        &built_in_noise_params::OFFSET,
                        None
                    )))
                    .into(),
                    WrapperType::Cache2D
                )
                .into(),
                WrapperType::FlatCache
            )
            .into();
        pub static ref SHIFT_Z: SharedComponentReference =
            WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                    ShiftBFunction::new(Arc::new(InternalNoise::new(
                        &built_in_noise_params::OFFSET,
                        None
                    )))
                    .into(),
                    WrapperType::Cache2D
                )
                .into(),
                WrapperType::FlatCache
            )
            .into();
        pub static ref BASE_3D_NOISE_OVERWORLD: SharedComponentReference =
            InterpolatedNoiseFunction::create_base_3d_noise_function(
                0.25f64, 0.125f64, 80f64, 160f64, 8f64
            )
            .into();
        pub static ref BASE_3D_NOISE_NETHER: SharedComponentReference =
            InterpolatedNoiseFunction::create_base_3d_noise_function(
                0.25f64, 0.375f64, 80f64, 60f64, 8f64
            )
            .into();
        pub static ref BASE_3D_NOISE_END: SharedComponentReference =
            InterpolatedNoiseFunction::create_base_3d_noise_function(
                0.25f64, 0.25f64, 80f64, 160f64, 4f64
            )
            .into();
        pub static ref CONTINENTS_OVERWORLD: SharedComponentReference =
            WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                ShiftedNoiseFunction::<
                    NoEnvironment,
                    SharedComponentReference,
                    SharedComponentReference,
                    SharedComponentReference,
                >::new(
                    SHIFT_X.clone(),
                    ZERO.clone(),
                    SHIFT_Z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(
                        &built_in_noise_params::CONTINENTALNESS,
                        None
                    ))
                )
                .into(),
                WrapperType::FlatCache
            )
            .into();
        pub static ref EROSION_OVERWORLD: SharedComponentReference =
            WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                ShiftedNoiseFunction::<
                    NoEnvironment,
                    SharedComponentReference,
                    SharedComponentReference,
                    SharedComponentReference,
                >::new(
                    SHIFT_X.clone(),
                    ZERO.clone(),
                    SHIFT_Z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(&built_in_noise_params::EROSION, None))
                )
                .into(),
                WrapperType::FlatCache
            )
            .into();
        pub static ref RIDGES_OVERWORLD: SharedComponentReference =
            WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                ShiftedNoiseFunction::<
                    NoEnvironment,
                    SharedComponentReference,
                    SharedComponentReference,
                    SharedComponentReference,
                >::new(
                    SHIFT_X.clone(),
                    ZERO.clone(),
                    SHIFT_Z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(&built_in_noise_params::RIDGE, None))
                )
                .into(),
                WrapperType::FlatCache
            )
            .into();
        pub static ref RIDGES_FOLDED_OVERWORLD: SharedComponentReference = RIDGES_OVERWORLD
            .clone()
            .abs()
            .add_const(-0.6666666666666666f64)
            .abs()
            .add_const(-0.3333333333333333f64)
            .mul_const(-3f64);
        static ref JAGGED_NOISE: SharedComponentReference = NoiseFunction::new(
            Arc::new(InternalNoise::new(&built_in_noise_params::JAGGED, None)),
            1500f64,
            0f64
        )
        .into();
        pub static ref OFFSET_OVERWORLD: SharedComponentReference = apply_blending(
            ConstantFunction::new(-0.50375f32 as f64).add(
                SplineFunction::<NoEnvironment, ImmutableSplineRef>::new(
                    create_offset_spline(
                        CONTINENTS_OVERWORLD.clone(),
                        EROSION_OVERWORLD.clone(),
                        RIDGES_FOLDED_OVERWORLD.clone(),
                        false,
                    )
                    .into()
                )
                .into()
            ),
            BLEND_OFFSET.clone(),
        );
        pub static ref FACTOR_OVERWORLD: SharedComponentReference = apply_blending(
            SplineFunction::<NoEnvironment, ImmutableSplineRef>::new(
                create_factor_spline(
                    CONTINENTS_OVERWORLD.clone(),
                    EROSION_OVERWORLD.clone(),
                    RIDGES_OVERWORLD.clone(),
                    RIDGES_FOLDED_OVERWORLD.clone(),
                    false,
                )
                .into()
            )
            .into(),
            TEN.clone(),
        );
        pub static ref JAGGEDNESS_OVERWORLD: SharedComponentReference = apply_blending(
            SplineFunction::<NoEnvironment, ImmutableSplineRef>::new(
                create_jaggedness_spline(
                    CONTINENTS_OVERWORLD.clone(),
                    EROSION_OVERWORLD.clone(),
                    RIDGES_OVERWORLD.clone(),
                    RIDGES_FOLDED_OVERWORLD.clone(),
                    false,
                )
                .into()
            )
            .into(),
            ZERO.clone(),
        );
        pub static ref DEPTH_OVERWORLD: SharedComponentReference =
            YClampedFunction::new(-64, 320, 1.5f64, -1.5f64).add(OFFSET_OVERWORLD.clone());
        pub static ref SLOPED_CHEESE_OVERWORLD: SharedComponentReference = {
            let density1 = JAGGEDNESS_OVERWORLD
                .clone()
                .mul(JAGGED_NOISE.clone().half_negative());
            let density2 = ConstantFunction::new(4f64).mul(
                DEPTH_OVERWORLD
                    .clone()
                    .add(density1)
                    .mul(FACTOR_OVERWORLD.clone())
                    .quarter_negative(),
            );

            density2.add(BASE_3D_NOISE_OVERWORLD.clone())
        };
        pub static ref CONTINENTS_OVERWORLD_LARGE_BIOME: SharedComponentReference =
            WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                ShiftedNoiseFunction::<
                    NoEnvironment,
                    SharedComponentReference,
                    SharedComponentReference,
                    SharedComponentReference,
                >::new(
                    SHIFT_X.clone(),
                    ZERO.clone(),
                    SHIFT_Z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(
                        &built_in_noise_params::CONTINENTALNESS_LARGE,
                        None
                    ))
                )
                .into(),
                WrapperType::FlatCache
            )
            .into();
        pub static ref EROSION_OVERWORLD_LARGE_BIOME: SharedComponentReference =
            WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                ShiftedNoiseFunction::<
                    NoEnvironment,
                    SharedComponentReference,
                    SharedComponentReference,
                    SharedComponentReference,
                >::new(
                    SHIFT_X.clone(),
                    ZERO.clone(),
                    SHIFT_Z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(
                        &built_in_noise_params::EROSION_LARGE,
                        None
                    ))
                )
                .into(),
                WrapperType::FlatCache
            )
            .into();
        pub static ref OFFSET_OVERWORLD_LARGE_BIOME: SharedComponentReference = apply_blending(
            ConstantFunction::new(-0.50375f32 as f64).add(
                SplineFunction::<NoEnvironment, ImmutableSplineRef>::new(
                    create_offset_spline(
                        CONTINENTS_OVERWORLD_LARGE_BIOME.clone(),
                        EROSION_OVERWORLD_LARGE_BIOME.clone(),
                        RIDGES_FOLDED_OVERWORLD.clone(),
                        false,
                    )
                    .into()
                )
                .into()
            ),
            BLEND_OFFSET.clone()
        );
        pub static ref FACTOR_OVERWORLD_LARGE_BIOME: SharedComponentReference = apply_blending(
            SplineFunction::<NoEnvironment, ImmutableSplineRef>::new(
                create_factor_spline(
                    CONTINENTS_OVERWORLD_LARGE_BIOME.clone(),
                    EROSION_OVERWORLD_LARGE_BIOME.clone(),
                    RIDGES_OVERWORLD.clone(),
                    RIDGES_FOLDED_OVERWORLD.clone(),
                    false,
                )
                .into()
            )
            .into(),
            TEN.clone()
        );
        pub static ref JAGGEDNESS_OVERWORLD_LARGE_BIOME: SharedComponentReference = apply_blending(
            SplineFunction::<NoEnvironment, ImmutableSplineRef>::new(
                create_jaggedness_spline(
                    CONTINENTS_OVERWORLD_LARGE_BIOME.clone(),
                    EROSION_OVERWORLD_LARGE_BIOME.clone(),
                    RIDGES_OVERWORLD.clone(),
                    RIDGES_FOLDED_OVERWORLD.clone(),
                    false,
                )
                .into()
            )
            .into(),
            ZERO.clone()
        );
        pub static ref DEPTH_OVERWORLD_LARGE_BIOME: SharedComponentReference =
            YClampedFunction::new(-64, 320, 1.5f64, -1.5f64)
                .add(OFFSET_OVERWORLD_LARGE_BIOME.clone());
        pub static ref SLOPED_CHEESE_OVERWORLD_LARGE_BIOME: SharedComponentReference = {
            let density1 = JAGGEDNESS_OVERWORLD_LARGE_BIOME
                .clone()
                .mul(JAGGED_NOISE.clone().half_negative());
            let density2 = ConstantFunction::new(4f64).mul(
                DEPTH_OVERWORLD_LARGE_BIOME
                    .clone()
                    .add(density1)
                    .mul(FACTOR_OVERWORLD_LARGE_BIOME.clone())
                    .quarter_negative(),
            );

            density2.add(BASE_3D_NOISE_OVERWORLD.clone())
        };
        pub static ref OFFSET_OVERWORLD_AMPLIFIED: SharedComponentReference = apply_blending(
            ConstantFunction::new(-0.50375f32 as f64).add(
                SplineFunction::<NoEnvironment, ImmutableSplineRef>::new(
                    create_offset_spline(
                        CONTINENTS_OVERWORLD.clone(),
                        EROSION_OVERWORLD.clone(),
                        RIDGES_FOLDED_OVERWORLD.clone(),
                        true
                    )
                    .into()
                )
                .into()
            ),
            BLEND_OFFSET.clone()
        );
        pub static ref FACTOR_OVERWORLD_AMPLIFIED: SharedComponentReference = apply_blending(
            SplineFunction::<NoEnvironment, ImmutableSplineRef>::new(
                create_factor_spline(
                    CONTINENTS_OVERWORLD.clone(),
                    EROSION_OVERWORLD.clone(),
                    RIDGES_OVERWORLD.clone(),
                    RIDGES_FOLDED_OVERWORLD.clone(),
                    true
                )
                .into()
            )
            .into(),
            TEN.clone()
        );
        pub static ref JAGGEDNESS_OVERWORLD_AMPLIFIED: SharedComponentReference = apply_blending(
            SplineFunction::<NoEnvironment, ImmutableSplineRef>::new(
                create_jaggedness_spline(
                    CONTINENTS_OVERWORLD.clone(),
                    EROSION_OVERWORLD.clone(),
                    RIDGES_OVERWORLD.clone(),
                    RIDGES_FOLDED_OVERWORLD.clone(),
                    true
                )
                .into()
            )
            .into(),
            ZERO.clone()
        );
        pub static ref DEPTH_OVERWORLD_AMPLIFIED: SharedComponentReference =
            YClampedFunction::new(-64, 320, 1.5f64, -1.5f64)
                .add(OFFSET_OVERWORLD_AMPLIFIED.clone());
        pub static ref SLOPED_CHEESE_OVERWORLD_AMPLIFIED: SharedComponentReference = {
            let density1 = JAGGEDNESS_OVERWORLD_AMPLIFIED
                .clone()
                .mul(JAGGED_NOISE.clone().half_negative());
            let density2 = ConstantFunction::new(4f64).mul(
                DEPTH_OVERWORLD_AMPLIFIED
                    .clone()
                    .add(density1)
                    .mul(FACTOR_OVERWORLD_AMPLIFIED.clone())
                    .quarter_negative(),
            );

            density2.add(BASE_3D_NOISE_OVERWORLD.clone())
        };
        pub static ref SLOPED_CHEESE_END: SharedComponentReference =
            EndIslandFunction::new(0).add(BASE_3D_NOISE_END.clone());
        pub static ref CAVES_SPAGHETTI_ROUGHNESS_FUNCTION_OVERWORLD: SharedComponentReference =
            WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                noise_in_range(
                    &built_in_noise_params::SPAGHETTI_ROUGHNESS_MODULATOR,
                    1f64,
                    1f64,
                    0f64,
                    -0.1f64
                )
                .mul(
                    Into::<SharedComponentReference>::into(NoiseFunction::new(
                        Arc::new(InternalNoise::new(
                            &built_in_noise_params::SPAGHETTI_ROUGHNESS,
                            None
                        )),
                        1f64,
                        1f64
                    ))
                    .abs()
                    .add_const(-0.4f64)
                ),
                WrapperType::OnceCache,
            )
            .into();
        pub static ref CAVES_SPAGHETTI_2D_THICKNESS_MODULAR_OVERWORLD: SharedComponentReference =
            WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                noise_in_range(
                    &built_in_noise_params::SPAGHETTI_2D_THICKNESS,
                    2f64,
                    1f64,
                    -0.6f64,
                    -1.3f64
                ),
                WrapperType::OnceCache,
            )
            .into();
        pub static ref CAVES_SPAGHETTI_2D_OVERWORLD: SharedComponentReference = {
            let function1 = NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    &built_in_noise_params::SPAGHETTI_2D_MODULATOR,
                    None,
                )),
                2f64,
                1f64,
            );

            let function2 = WierdScaledFunction::<NoEnvironment, SharedComponentReference>::new(
                function1.into(),
                Arc::new(InternalNoise::new(
                    &built_in_noise_params::SPAGHETTI_2D,
                    None,
                )),
                RarityMapper::Caves,
            );

            let function3 = noise_in_range(
                &built_in_noise_params::SPAGHETTI_2D_ELEVATION,
                1f64,
                0f64,
                ((-64i32) / 8i32) as f64,
                8f64,
            );

            let function4 = CAVES_SPAGHETTI_2D_THICKNESS_MODULAR_OVERWORLD.clone();

            let function5 = function3.add(YClampedFunction::new(-64, 320, 8f64, -40f64).abs());

            let function6 = function5.add(function4.clone()).cube();

            let function7 = function2.add(function4.mul_const(0.083f64));

            function7.max(function6).clamp(-1f64, 1f64)
        };
        pub static ref CAVES_ENTRANCES_OVERWORLD: SharedComponentReference = {
            let function_ref: SharedComponentReference =
                WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                    NoiseFunction::new(
                        Arc::new(InternalNoise::new(
                            &built_in_noise_params::SPAGHETTI_3D_RARITY,
                            None,
                        )),
                        2f64,
                        1f64,
                    )
                    .into(),
                    WrapperType::OnceCache,
                )
                .into();

            let function2 = noise_in_range(
                &built_in_noise_params::SPAGHETTI_3D_THICKNESS,
                1f64,
                1f64,
                -0.065f64,
                -0.088f64,
            );

            let function3 = WierdScaledFunction::<NoEnvironment, SharedComponentReference>::new(
                function_ref.clone(),
                Arc::new(InternalNoise::new(
                    &built_in_noise_params::SPAGHETTI_3D_1,
                    None,
                )),
                RarityMapper::Tunnels,
            );

            let function4 = WierdScaledFunction::<NoEnvironment, SharedComponentReference>::new(
                function_ref,
                Arc::new(InternalNoise::new(
                    &built_in_noise_params::SPAGHETTI_3D_2,
                    None,
                )),
                RarityMapper::Tunnels,
            );
            let function5 = function3
                .max(function4.into())
                .add(function2)
                .clamp(-1f64, 1f64);

            let function6 = CAVES_SPAGHETTI_ROUGHNESS_FUNCTION_OVERWORLD.clone();

            let function7 = NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    &built_in_noise_params::CAVE_ENTRANCE,
                    None,
                )),
                0.75f64,
                0.5f64,
            );

            let function8 = function7
                .add_const(0.37f64)
                .add(YClampedFunction::new(-10, 30, 0.3f64, 0f64).into());

            WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                function8.min(function6.add(function5)),
                WrapperType::OnceCache,
            )
            .into()
        };
        pub static ref CAVES_NOODLE_OVERWORLD: SharedComponentReference = {
            let function2 = vertical_range_choice(
                Y.clone(),
                NoiseFunction::new(
                    Arc::new(InternalNoise::new(&built_in_noise_params::NOODLE, None)),
                    1f64,
                    1f64,
                )
                .into(),
                -60,
                320,
                -1,
            );

            let function3 = vertical_range_choice(
                Y.clone(),
                noise_in_range(
                    &built_in_noise_params::NOODLE_THICKNESS,
                    1f64,
                    1f64,
                    -0.05f64,
                    -0.1f64,
                ),
                -60,
                320,
                0,
            );

            let function4 = vertical_range_choice(
                Y.clone(),
                NoiseFunction::new(
                    Arc::new(InternalNoise::new(
                        &built_in_noise_params::NOODLE_RIDGE_A,
                        None,
                    )),
                    2.6666666666666665f64,
                    2.6666666666666665f64,
                )
                .into(),
                -60,
                320,
                0,
            );

            let function5 = vertical_range_choice(
                Y.clone(),
                NoiseFunction::new(
                    Arc::new(InternalNoise::new(
                        &built_in_noise_params::NOODLE_RIDGE_B,
                        None,
                    )),
                    2.6666666666666665f64,
                    2.6666666666666665f64,
                )
                .into(),
                -60,
                320,
                0,
            );

            let function6 = function4.abs().max(function5.abs()).mul_const(1.5f64);

            RangeFunction::<
                NoEnvironment,
                SharedComponentReference,
                SharedComponentReference,
                SharedComponentReference,
            >::new(
                function2,
                -1000000f64,
                0f64,
                ConstantFunction::new(64f64).into(),
                function3.add(function6),
            )
            .into()
        };
        pub static ref CAVES_PILLARS_OVERWORLD: SharedComponentReference = {
            let function = NoiseFunction::new(
                Arc::new(InternalNoise::new(&built_in_noise_params::PILLAR, None)),
                25f64,
                0.3f64,
            );

            let function2 = noise_in_range(
                &built_in_noise_params::PILLAR_RARENESS,
                1f64,
                1f64,
                0f64,
                -2f64,
            );

            let function3 = noise_in_range(
                &built_in_noise_params::PILLAR_THICKNESS,
                1f64,
                1f64,
                0f64,
                1.1f64,
            );

            let function4 = function.mul_const(2f64).add(function2);

            WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                function4.mul(function3.cube()),
                WrapperType::OnceCache,
            )
            .into()
        };
    }
}

pub fn peaks_valleys_noise(variance: f32) -> f32 {
    -((variance.abs() - 0.6666667f32).abs() - 0.33333334f32) * 3f32
}

pub fn vertical_range_choice(
    input: SharedComponentReference,
    in_range: SharedComponentReference,
    min: i32,
    max: i32,
    out: i32,
) -> SharedComponentReference {
    WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
        RangeFunction::<
            NoEnvironment,
            SharedComponentReference,
            SharedComponentReference,
            SharedComponentReference,
        >::new(
            input,
            min as f64,
            (max + 1) as f64,
            in_range,
            ConstantFunction::new(out as f64).into(),
        )
        .into(),
        WrapperType::Interpolated,
    )
    .into()
}

pub fn apply_blend_density(density: SharedComponentReference) -> SharedComponentReference {
    let function = BlendDensityFunction::<NoEnvironment, SharedComponentReference>::new(density);
    Into::<SharedComponentReference>::into(
        WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
            function.into(),
            WrapperType::Interpolated,
        ),
    )
    .mul_const(0.64f64)
    .squeeze()
}

fn apply_blending(
    function: SharedComponentReference,
    blend: SharedComponentReference,
) -> SharedComponentReference {
    let function = lerp_density(BLEND_ALPHA.clone(), blend, function);

    WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
        WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
            function,
            WrapperType::Cache2D,
        )
        .into(),
        WrapperType::FlatCache,
    )
    .into()
}

fn noise_in_range(
    noise: &'static DoublePerlinNoiseParameters,
    xz_scale: f64,
    y_scale: f64,
    min: f64,
    max: f64,
) -> SharedComponentReference {
    map_range(
        NoiseFunction::new(Arc::new(InternalNoise::new(noise, None)), xz_scale, y_scale).into(),
        min,
        max,
    )
}

fn map_range(function: SharedComponentReference, min: f64, max: f64) -> SharedComponentReference {
    let d = (min + max) * 0.5f64;
    let e = (max - min) * 0.5f64;

    function.mul_const(e).add_const(d)
}

pub fn lerp_density(
    delta: SharedComponentReference,
    start: SharedComponentReference,
    end: SharedComponentReference,
) -> SharedComponentReference {
    if let ConverterEnvironment::<NoEnvironment>::Constant(constant) = start.environment() {
        lerp_density_static_start(delta, constant, end)
    } else {
        let function_ref: SharedComponentReference = WrapperFunction::<
            NoEnvironment,
            SharedComponentReference,
        >::new(delta, WrapperType::OnceCache)
        .into();

        let function2 = function_ref.clone().mul_const(-1f64).add_const(1f64);
        start.mul(function2).add(end.mul(function_ref))
    }
}

pub fn lerp_density_static_start(
    delta: SharedComponentReference,
    start: f64,
    end: SharedComponentReference,
) -> SharedComponentReference {
    delta.mul(end.add_const(-start)).add_const(start)
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use pumpkin_core::random::{
        legacy_rand::LegacyRand, RandomDeriver, RandomDeriverImpl, RandomImpl,
    };

    use crate::world_gen::noise::{
        built_in_noise_params,
        density::{built_in_density_function::*, NoisePos, UnblendedNoisePos},
        perlin::DoublePerlinNoiseSampler,
    };

    use super::{
        component_functions::{
            ComponentFunctionImpl, ComponentReference, ComponentReferenceImplementation,
            ComponentReferenceMath, ConversionResultPre, ConverterEnvironment, ConverterImpl,
            DensityFunctionEnvironment, EnvironmentApplierImpl, MutableComponentFunctionImpl,
            MutableComponentReference, NoEnvironment, OwnedConverterEnvironment,
            SharedComponentReference,
        },
        noise::{InternalNoise, NoiseFunction},
    };

    pub struct TestConverter {
        pub splitter: RandomDeriver,
    }

    impl ConverterImpl<NoEnvironment> for TestConverter {
        fn convert_noise(&mut self, noise: &Arc<InternalNoise>) -> Option<Arc<InternalNoise>> {
            let id = noise.parameters.id();
            let mut rand = self.splitter.split_string(id);
            let sampler = DoublePerlinNoiseSampler::new(&mut rand, noise.parameters, false);

            Some(Arc::new(InternalNoise::new(
                noise.parameters,
                Some(sampler),
            )))
        }

        fn convert_env_pre_internal(
            &mut self,
            _component: ConverterEnvironment<NoEnvironment>,
        ) -> ConversionResultPre<NoEnvironment> {
            ConversionResultPre::Default
        }

        fn converts_post_internal(
            &mut self,
            _component: ConverterEnvironment<NoEnvironment>,
        ) -> bool {
            false
        }

        fn convert_env_post_internal(
            &mut self,
            _component: OwnedConverterEnvironment<NoEnvironment>,
        ) -> ComponentReferenceImplementation<NoEnvironment> {
            unreachable!()
        }
    }

    #[test]
    fn test_density_function_correctness() {
        let pos = NoisePos::Unblended(UnblendedNoisePos { x: 0, y: 0, z: 0 });

        assert_eq!(BLEND_ALPHA.sample(&pos), 1f64);

        assert_eq!(BLEND_OFFSET.sample(&pos), 0f64);

        assert_eq!(ZERO.sample(&pos), 0f64);

        assert_eq!(Y.sample(&pos), 0f64);

        assert_eq!(SHIFT_X.sample(&pos), 0f64);

        assert_eq!(SHIFT_Z.sample(&pos), 0f64);

        assert_eq!(BASE_3D_NOISE_OVERWORLD.sample(&pos), 0.05283727086562935f64);

        assert_eq!(BASE_3D_NOISE_NETHER.sample(&pos), 0.05283727086562935f64);

        assert_eq!(BASE_3D_NOISE_END.sample(&pos), 0.05283727086562935f64);

        assert_eq!(CONTINENTS_OVERWORLD.sample(&pos), 0f64);

        assert_eq!(EROSION_OVERWORLD.sample(&pos), 0f64);

        assert_eq!(RIDGES_OVERWORLD.sample(&pos), 0f64);

        assert_eq!(RIDGES_FOLDED_OVERWORLD.sample(&pos), -1f64);

        assert_eq!(OFFSET_OVERWORLD.sample(&pos), -0.6037500277161598f64);

        assert_eq!(FACTOR_OVERWORLD.sample(&pos), 5.549900531768799f64);

        assert_eq!(JAGGEDNESS_OVERWORLD.sample(&pos), 0f64);

        assert_eq!(DEPTH_OVERWORLD.sample(&pos), 0.3962499722838402f64);

        assert_eq!(SLOPED_CHEESE_OVERWORLD.sample(&pos), 8.849428998431454f64);

        assert_eq!(CONTINENTS_OVERWORLD_LARGE_BIOME.sample(&pos), 0f64);

        assert_eq!(EROSION_OVERWORLD_LARGE_BIOME.sample(&pos), 0f64);

        assert_eq!(
            OFFSET_OVERWORLD_LARGE_BIOME.sample(&pos),
            -0.6037500277161598f64
        );

        assert_eq!(
            FACTOR_OVERWORLD_LARGE_BIOME.sample(&pos),
            5.549900531768799f64
        );

        assert_eq!(JAGGEDNESS_OVERWORLD_LARGE_BIOME.sample(&pos), 0f64);

        assert_eq!(
            DEPTH_OVERWORLD_LARGE_BIOME.sample(&pos),
            0.3962499722838402f64
        );

        assert_eq!(
            SLOPED_CHEESE_OVERWORLD_LARGE_BIOME.sample(&pos),
            8.849428998431454f64
        );

        assert_eq!(
            OFFSET_OVERWORLD_AMPLIFIED.sample(&pos),
            -0.6037500277161598f64
        );

        assert_eq!(
            FACTOR_OVERWORLD_AMPLIFIED.sample(&pos),
            0.6516130566596985f64
        );

        assert_eq!(JAGGEDNESS_OVERWORLD_AMPLIFIED.sample(&pos), 0f64);

        assert_eq!(
            DEPTH_OVERWORLD_AMPLIFIED.sample(&pos),
            0.3962499722838402f64
        );

        assert_eq!(
            SLOPED_CHEESE_OVERWORLD_AMPLIFIED.sample(&pos),
            1.085643893430405f64
        );

        assert_eq!(SLOPED_CHEESE_END.sample(&pos), 0.6153372708656294f64);

        assert_eq!(
            CAVES_SPAGHETTI_ROUGHNESS_FUNCTION_OVERWORLD.sample(&pos),
            0.020000000000000004f64
        );

        assert_eq!(
            CAVES_ENTRANCES_OVERWORLD.sample(&pos),
            -0.056499999999999995f64
        );

        assert_eq!(CAVES_NOODLE_OVERWORLD.sample(&pos), -0.07500000000000001f64);

        assert_eq!(
            CAVES_PILLARS_OVERWORLD.sample(&pos),
            -0.16637500000000005f64
        );

        assert_eq!(
            CAVES_SPAGHETTI_2D_THICKNESS_MODULAR_OVERWORLD.sample(&pos),
            -0.95f64
        );

        assert_eq!(CAVES_SPAGHETTI_2D_OVERWORLD.sample(&pos), -0.07885f64);
    }

    #[test]
    fn test_conversion() {
        let pos = &NoisePos::Unblended(UnblendedNoisePos::new(0, 0, 0));

        let mut rand = LegacyRand::from_seed(0);
        let splitter = rand.next_splitter();
        let mut rand2 = splitter.split_string("test");
        assert_eq!(rand2.next_i32(), 457448999);

        let mut converter = TestConverter {
            splitter: RandomDeriver::Legacy(splitter),
        };

        let shift_x = SHIFT_X.clone();
        assert_eq!(shift_x.sample(pos), 0f64);

        let converted_shift_x = shift_x.clone().convert(&mut converter).assert_shared();
        assert_eq!(converted_shift_x.sample(pos), -1.0434329952492611f64);

        let converted = ZERO.clone().convert(&mut converter).assert_shared();
        assert_eq!(converted.sample(pos), 0f64);

        let converted = Y.clone().convert(&mut converter).assert_shared();
        assert_eq!(converted.sample(pos), 0f64);

        let converted = SHIFT_Z.clone().convert(&mut converter).assert_shared();
        assert_eq!(converted.sample(pos), -1.0434329952492611f64);

        let converted = BASE_3D_NOISE_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0.05283727086562935f64);

        let converted = BASE_3D_NOISE_NETHER
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0.05283727086562935f64);

        let converted = BASE_3D_NOISE_END
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0.05283727086562935f64);

        let converted = CONTINENTS_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), -0.07789864655134425f64);

        let converted = EROSION_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), -0.1909838546072366f64);

        let converted = RIDGES_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), -0.18402646504201148f64);

        let converted = RIDGES_FOLDED_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), -0.4479206048739655f64);

        let converted = OFFSET_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), -0.4883981039747596f64);

        let converted = FACTOR_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 5.053797245025635f64);

        let converted = JAGGEDNESS_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0f64);

        let converted = DEPTH_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0.5116018960252404f64);

        let converted = SLOPED_CHEESE_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 10.394966281594634f64);

        let converted = CONTINENTS_OVERWORLD_LARGE_BIOME
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0.5413000828801735f64);

        let converted = EROSION_OVERWORLD_LARGE_BIOME
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), -0.22584626355586912f64);

        let converted = OFFSET_OVERWORLD_LARGE_BIOME
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), -0.3040638118982315f64);

        let converted = FACTOR_OVERWORLD_LARGE_BIOME
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 6.040968418121338f64);

        let converted = JAGGEDNESS_OVERWORLD_LARGE_BIOME
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0f64);

        let converted = DEPTH_OVERWORLD_LARGE_BIOME
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0.6959361881017685f64);

        let converted = SLOPED_CHEESE_OVERWORLD_LARGE_BIOME
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 16.869351404267768f64);

        let converted = OFFSET_OVERWORLD_AMPLIFIED
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), -0.47187428921461105f64);

        let converted = FACTOR_OVERWORLD_AMPLIFIED
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0.6070870161056519f64);

        let converted = JAGGEDNESS_OVERWORLD_AMPLIFIED
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0f64);

        let converted = DEPTH_OVERWORLD_AMPLIFIED
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0.528125710785389f64);

        let converted = SLOPED_CHEESE_OVERWORLD_AMPLIFIED
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 1.3353103184231423f64);

        let converted = SLOPED_CHEESE_END
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0.6153372708656294f64);

        let converted = CAVES_SPAGHETTI_ROUGHNESS_FUNCTION_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0.0022723587537428667f64);

        let converted = CAVES_ENTRANCES_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 0.20826666534765442f64);

        let converted = CAVES_NOODLE_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 64f64);

        let converted = CAVES_PILLARS_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), -0.0863817754261902f64);

        let converted = CAVES_SPAGHETTI_2D_THICKNESS_MODULAR_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), -0.9871490512034798f64);

        let converted = CAVES_SPAGHETTI_2D_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();
        assert_eq!(converted.sample(pos), 1f64);
    }

    pub struct OwnedConverter {
        pub splitter: RandomDeriver,
    }

    struct MutableWrapper {
        wrapped: SharedComponentReference,
    }

    impl ComponentFunctionImpl for MutableWrapper {}

    impl<E: DensityFunctionEnvironment> MutableComponentFunctionImpl<E> for MutableWrapper {
        fn sample_mut(&mut self, pos: &NoisePos, _env: &E) -> f64 {
            self.wrapped.sample(pos)
        }

        fn fill_mut(&mut self, arr: &mut [f64], applier: &mut dyn EnvironmentApplierImpl<Env = E>) {
            self.wrapped.fill(arr, applier.cast_up());
        }

        fn environment(&self) -> ConverterEnvironment<E> {
            // TODO: actually test owned conversion?
            unreachable!()
        }

        fn into_environment(self: Box<Self>) -> OwnedConverterEnvironment<E> {
            unreachable!()
        }

        fn convert(
            self: Box<Self>,
            _converter: &mut dyn ConverterImpl<E>,
        ) -> ComponentReferenceImplementation<E> {
            ComponentReferenceImplementation::Mutable(MutableComponentReference(self))
        }

        fn clone_to_new_ref(&self) -> ComponentReferenceImplementation<E> {
            unimplemented!()
        }
    }

    pub struct FakeEnvironment {}

    impl DensityFunctionEnvironment for FakeEnvironment {}

    impl ConverterImpl<FakeEnvironment> for OwnedConverter {
        fn convert_noise(&mut self, noise: &Arc<InternalNoise>) -> Option<Arc<InternalNoise>> {
            let id = noise.parameters.id();
            let mut rand = self.splitter.split_string(id);
            let sampler = DoublePerlinNoiseSampler::new(&mut rand, noise.parameters, false);

            Some(Arc::new(InternalNoise::new(
                noise.parameters,
                Some(sampler),
            )))
        }

        fn converts_post_internal(
            &mut self,
            _component: ConverterEnvironment<FakeEnvironment>,
        ) -> bool {
            false
        }

        fn convert_env_post_internal(
            &mut self,
            _component: OwnedConverterEnvironment<FakeEnvironment>,
        ) -> ComponentReferenceImplementation<FakeEnvironment> {
            unreachable!()
        }

        fn convert_env_pre_internal(
            &mut self,
            _component: ConverterEnvironment<FakeEnvironment>,
        ) -> ConversionResultPre<FakeEnvironment> {
            ConversionResultPre::Default
        }
    }

    impl SharedComponentReference {
        pub fn convert_to_dyn<E: DensityFunctionEnvironment>(
            self,
            converter: &mut dyn ConverterImpl<E>,
        ) -> Box<dyn ComponentReference<E>> {
            match self.convert(converter) {
                ComponentReferenceImplementation::Shared(shared) => Box::new(shared),
                ComponentReferenceImplementation::Mutable(owned) => Box::new(owned),
            }
        }
    }

    #[test]
    fn test_owned_converter() {
        let test_func = NoiseFunction::new(
            Arc::new(InternalNoise::new(
                &built_in_noise_params::NETHER_WART,
                None,
            )),
            10f64,
            1.2f64,
        )
        .add(
            NoiseFunction::new(
                Arc::new(InternalNoise::new(&built_in_noise_params::NOODLE, None)),
                0.1f64,
                -1f64,
            )
            .into(),
        );

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
    fn test_owned_conversion() {
        let pos = &NoisePos::Unblended(UnblendedNoisePos::new(0, 0, 0));

        let mut rand = LegacyRand::from_seed(0);
        let splitter = rand.next_splitter();
        let mut rand2 = splitter.split_string("test");
        assert_eq!(rand2.next_i32(), 457448999);

        let mut converter = OwnedConverter {
            splitter: RandomDeriver::Legacy(splitter),
        };

        let env = &FakeEnvironment {};

        let shift_x = SHIFT_X.clone();
        assert_eq!(shift_x.sample(pos), 0f64);

        let mut converted_shift_x = shift_x.clone().convert_to_dyn(&mut converter);
        assert_eq!(
            converted_shift_x.sample_mut(pos, env),
            -1.0434329952492611f64
        );

        let mut converted = ZERO.clone().convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0f64);

        let mut converted = Y.clone().convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0f64);

        let mut converted = SHIFT_Z.clone().convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), -1.0434329952492611f64);

        let mut converted = BASE_3D_NOISE_OVERWORLD
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0.05283727086562935f64);

        let mut converted = BASE_3D_NOISE_NETHER.clone().convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0.05283727086562935f64);

        let mut converted = BASE_3D_NOISE_END.clone().convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0.05283727086562935f64);

        let mut converted = CONTINENTS_OVERWORLD.clone().convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), -0.07789864655134425f64);

        let mut converted = EROSION_OVERWORLD.clone().convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), -0.1909838546072366f64);

        let mut converted = RIDGES_OVERWORLD.clone().convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), -0.18402646504201148f64);

        let mut converted = RIDGES_FOLDED_OVERWORLD
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), -0.4479206048739655f64);

        let mut converted = OFFSET_OVERWORLD.clone().convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), -0.4883981039747596f64);

        let mut converted = FACTOR_OVERWORLD.clone().convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 5.053797245025635f64);

        let mut converted = JAGGEDNESS_OVERWORLD.clone().convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0f64);

        let mut converted = DEPTH_OVERWORLD.clone().convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0.5116018960252404f64);

        let mut converted = SLOPED_CHEESE_OVERWORLD
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 10.394966281594634f64);

        let mut converted = CONTINENTS_OVERWORLD_LARGE_BIOME
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0.5413000828801735f64);

        let mut converted = EROSION_OVERWORLD_LARGE_BIOME
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), -0.22584626355586912f64);

        let mut converted = OFFSET_OVERWORLD_LARGE_BIOME
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), -0.3040638118982315f64);

        let mut converted = FACTOR_OVERWORLD_LARGE_BIOME
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 6.040968418121338f64);

        let mut converted = JAGGEDNESS_OVERWORLD_LARGE_BIOME
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0f64);

        let mut converted = DEPTH_OVERWORLD_LARGE_BIOME
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0.6959361881017685f64);

        let mut converted = SLOPED_CHEESE_OVERWORLD_LARGE_BIOME
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 16.869351404267768f64);

        let mut converted = OFFSET_OVERWORLD_AMPLIFIED
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), -0.47187428921461105f64);

        let mut converted = FACTOR_OVERWORLD_AMPLIFIED
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0.6070870161056519f64);

        let mut converted = JAGGEDNESS_OVERWORLD_AMPLIFIED
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0f64);

        let mut converted = DEPTH_OVERWORLD_AMPLIFIED
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0.528125710785389f64);

        let mut converted = SLOPED_CHEESE_OVERWORLD_AMPLIFIED
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 1.3353103184231423f64);

        let mut converted = SLOPED_CHEESE_END.clone().convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0.6153372708656294f64);

        let mut converted = CAVES_SPAGHETTI_ROUGHNESS_FUNCTION_OVERWORLD
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0.0022723587537428667f64);

        let mut converted = CAVES_ENTRANCES_OVERWORLD
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 0.20826666534765442f64);

        let mut converted = CAVES_NOODLE_OVERWORLD
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 64f64);

        let mut converted = CAVES_PILLARS_OVERWORLD
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), -0.0863817754261902f64);

        let mut converted = CAVES_SPAGHETTI_2D_THICKNESS_MODULAR_OVERWORLD
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), -0.9871490512034798f64);

        let mut converted = CAVES_SPAGHETTI_2D_OVERWORLD
            .clone()
            .convert_to_dyn(&mut converter);
        assert_eq!(converted.sample_mut(pos, env), 1f64);
    }
}
