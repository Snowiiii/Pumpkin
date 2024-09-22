use std::{ops::Deref, sync::Arc};

use blend::{BlendAlphaFunction, BlendDensityFunction, BlendOffsetFunction};
use end::EndIslandFunction;
use enum_dispatch::enum_dispatch;
use math::{BinaryFunction, BinaryType, LinearFunction};
use noise::{InternalNoise, InterpolatedNoiseSampler, NoiseFunction, ShiftedNoiseFunction};
use offset::{ShiftAFunction, ShiftBFunction};
use spline::SplineFunction;
use unary::{ClampFunction, UnaryFunction, UnaryType};
use weird::WierdScaledFunction;

use crate::world_gen::blender::{Blender, NoBlendBlender};

use super::{
    chunk_sampler::{ChunkNoiseSampler, FlatCacheDensityFunction, InterpolatorDensityFunction},
    clamped_map,
    perlin::DoublePerlinNoiseParameters,
};

mod blend;
mod end;
mod math;
pub mod noise;
mod offset;
pub mod spline;
mod unary;
mod weird;

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
        apply_blending,
        blend::{BlendAlphaFunction, BlendOffsetFunction},
        end::EndIslandFunction,
        noise::{InternalNoise, InterpolatedNoiseSampler, NoiseFunction, ShiftedNoiseFunction},
        noise_in_range,
        offset::{ShiftAFunction, ShiftBFunction},
        spline::SplineFunction,
        veritcal_range_choice,
        weird::{RarityMapper, WierdScaledFunction},
        ConstantFunction, DensityFunction, RangeFunction, WrapperFunction, WrapperType,
        YClampedFunction,
    };

    pub struct SlopedCheeseResult<'a> {
        pub(crate) offset: Arc<DensityFunction<'a>>,
        pub(crate) factor: Arc<DensityFunction<'a>>,
        pub(crate) depth: Arc<DensityFunction<'a>>,
        pub(crate) jaggedness: Arc<DensityFunction<'a>>,
        pub(crate) sloped_cheese: Arc<DensityFunction<'a>>,
    }

    type BuiltInNoise = LazyLock<Arc<DensityFunction<'static>>>;
    type BuiltInSlopedCheese = LazyLock<SlopedCheeseResult<'static>>;

    pub static BLEND_ALPHA: BuiltInNoise =
        LazyLock::new(|| Arc::new(DensityFunction::BlendAlpha(BlendAlphaFunction {})));

    pub static BLEND_OFFSET: BuiltInNoise =
        LazyLock::new(|| Arc::new(DensityFunction::BlendOffset(BlendOffsetFunction {})));

    pub static ZERO: BuiltInNoise =
        LazyLock::new(|| Arc::new(DensityFunction::Constant(ConstantFunction::new(0f64))));

    pub static TEN: BuiltInNoise =
        LazyLock::new(|| Arc::new(DensityFunction::Constant(ConstantFunction::new(10f64))));

    pub static Y: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::ClampedY(YClampedFunction {
                from: MIN_HEIGHT * 2,
                to: MAX_COLUMN_HEIGHT * 2,
                from_val: (MIN_HEIGHT * 2) as f64,
                to_val: (MAX_COLUMN_HEIGHT * 2) as f64,
            })
        })
    });

    pub static SHIFT_X: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
                    Arc::new(DensityFunction::ShiftA(ShiftAFunction::new(Arc::new(
                        InternalNoise::new(builtin_noise_params::OFFSET.clone(), None),
                    )))),
                    WrapperType::Cache2D,
                ))),
                WrapperType::CacheFlat,
            ))
        })
    });

    pub static SHIFT_Z: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
                    Arc::new(DensityFunction::ShiftB(ShiftBFunction::new(Arc::new(
                        InternalNoise::new(builtin_noise_params::OFFSET.clone(), None),
                    )))),
                    WrapperType::Cache2D,
                ))),
                WrapperType::CacheFlat,
            ))
        })
    });

    pub static BASE_3D_NOISE_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::InterpolatedNoise(
                InterpolatedNoiseSampler::create_base_3d_noise_function(
                    0.25f64, 0.125f64, 80f64, 160f64, 8f64,
                ),
            )
        })
    });

    pub static BASE_3D_NOISE_NETHER: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::InterpolatedNoise(
                InterpolatedNoiseSampler::create_base_3d_noise_function(
                    0.25f64, 0.375f64, 80f64, 60f64, 8f64,
                ),
            )
        })
    });

    pub static BASE_3D_NOISE_END: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::InterpolatedNoise(
                InterpolatedNoiseSampler::create_base_3d_noise_function(
                    0.25f64, 0.25f64, 80f64, 160f64, 4f64,
                ),
            )
        })
    });

    pub static CONTINENTS_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
                    SHIFT_X.clone(),
                    ZERO.clone(),
                    SHIFT_Z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(
                        builtin_noise_params::CONTINENTALNESS.clone(),
                        None,
                    )),
                ))),
                WrapperType::CacheFlat,
            ))
        })
    });

    pub static EROSION_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
                    SHIFT_X.clone(),
                    ZERO.clone(),
                    SHIFT_Z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(
                        builtin_noise_params::EROSION.clone(),
                        None,
                    )),
                ))),
                WrapperType::CacheFlat,
            ))
        })
    });

    pub static RIDGES_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
                    SHIFT_X.clone(),
                    ZERO.clone(),
                    SHIFT_Z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(
                        builtin_noise_params::RIDGE.clone(),
                        None,
                    )),
                ))),
                WrapperType::CacheFlat,
            ))
        })
    });

    pub static RIDGES_FOLDED_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            RIDGES_OVERWORLD
                .abs()
                .add_const(-0.6666666666666666f64)
                .abs()
                .add_const(-0.3333333333333333f64)
                .mul_const(-3f64)
        })
    });

    pub static OVERWORLD_SLOPED_CHEESE: BuiltInSlopedCheese = LazyLock::new(|| {
        sloped_cheese_function(
            Arc::new(DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    builtin_noise_params::JAGGED.clone(),
                    None,
                )),
                1500f64,
                0f64,
            ))),
            CONTINENTS_OVERWORLD.clone(),
            EROSION_OVERWORLD.clone(),
            RIDGES_OVERWORLD.clone(),
            RIDGES_FOLDED_OVERWORLD.clone(),
            false,
        )
    });

    pub static CONTINENTS_OVERWORLD_LARGE_BIOME: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
                    SHIFT_X.clone(),
                    ZERO.clone(),
                    SHIFT_Z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(
                        builtin_noise_params::CONTINENTALNESS_LARGE.clone(),
                        None,
                    )),
                ))),
                WrapperType::CacheFlat,
            ))
        })
    });

    pub static EROSION_OVERWORLD_LARGE_BIOME: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
                    SHIFT_X.clone(),
                    ZERO.clone(),
                    SHIFT_Z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(
                        builtin_noise_params::EROSION_LARGE.clone(),
                        None,
                    )),
                ))),
                WrapperType::CacheFlat,
            ))
        })
    });

    pub static OVERWORLD_LARGE_SLOPED_CHEESE: BuiltInSlopedCheese = LazyLock::new(|| {
        sloped_cheese_function(
            Arc::new(DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    builtin_noise_params::JAGGED.clone(),
                    None,
                )),
                1500f64,
                0f64,
            ))),
            CONTINENTS_OVERWORLD_LARGE_BIOME.clone(),
            EROSION_OVERWORLD_LARGE_BIOME.clone(),
            RIDGES_OVERWORLD.clone(),
            RIDGES_FOLDED_OVERWORLD.clone(),
            false,
        )
    });

    pub static OVERWORLD_AMPLIFIED_SLOPED_CHEESE: BuiltInSlopedCheese = LazyLock::new(|| {
        sloped_cheese_function(
            Arc::new(DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    builtin_noise_params::JAGGED.clone(),
                    None,
                )),
                1500f64,
                0f64,
            ))),
            CONTINENTS_OVERWORLD.clone(),
            EROSION_OVERWORLD.clone(),
            RIDGES_OVERWORLD.clone(),
            RIDGES_FOLDED_OVERWORLD.clone(),
            true,
        )
    });

    pub static SLOPED_CHEESE_END: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::EndIsland(EndIslandFunction::new(0)).add(BASE_3D_NOISE_END.clone())
        })
    });

    pub static CAVES_SPAGHETTI_ROUGHNESS_FUNCTION_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(
                    noise_in_range(
                        builtin_noise_params::SPAGHETTI_ROUGHNESS_MODULATOR.clone(),
                        1f64,
                        1f64,
                        0f64,
                        -0.1f64,
                    )
                    .mul(Arc::new(
                        DensityFunction::Noise(NoiseFunction::new(
                            Arc::new(InternalNoise::new(
                                builtin_noise_params::SPAGHETTI_ROUGHNESS.clone(),
                                None,
                            )),
                            1f64,
                            1f64,
                        ))
                        .abs()
                        .add_const(-0.4f64),
                    )),
                ),
                WrapperType::CacheOnce,
            ))
        })
    });

    pub static CAVES_SPAGHETTI_2D_THICKNESS_MODULAR_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(noise_in_range(
                    builtin_noise_params::SPAGHETTI_2D_THICKNESS.clone(),
                    2f64,
                    1f64,
                    -0.6f64,
                    -1.3f64,
                )),
                WrapperType::CacheOnce,
            ))
        })
    });

    pub static CAVES_SPAGHETTI_2D_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            let function1 = DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    builtin_noise_params::SPAGHETTI_2D_MODULATOR.clone(),
                    None,
                )),
                2f64,
                1f64,
            ));

            let function2 = DensityFunction::Wierd(WierdScaledFunction::new(
                Arc::new(function1),
                Arc::new(InternalNoise::new(
                    builtin_noise_params::SPAGHETTI_2D.clone(),
                    None,
                )),
                RarityMapper::Caves,
            ));

            let function3 = noise_in_range(
                builtin_noise_params::SPAGHETTI_2D_ELEVATION.clone(),
                1f64,
                0f64,
                ((-64i32) / 8i32) as f64,
                8f64,
            );

            let function4 = CAVES_SPAGHETTI_2D_THICKNESS_MODULAR_OVERWORLD.clone();

            let function5 = function3.add(Arc::new(
                DensityFunction::ClampedY(YClampedFunction {
                    from: -64,
                    to: 320,
                    from_val: 8f64,
                    to_val: -40f64,
                })
                .abs(),
            ));

            let function6 = Arc::new(function5.add(function4.clone()).cube());

            let function7 = function2.add(Arc::new(function4.mul_const(0.083f64)));

            function7.binary_max(function6).clamp(-1f64, 1f64)
        })
    });

    pub static CAVES_ENTRANCES_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            let function = DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    builtin_noise_params::SPAGHETTI_3D_RARITY.clone(),
                    None,
                )),
                2f64,
                1f64,
            ));

            let function2 = Arc::new(noise_in_range(
                builtin_noise_params::SPAGHETTI_3D_THICKNESS.clone(),
                1f64,
                1f64,
                -0.065f64,
                -0.088f64,
            ));

            let function3 = DensityFunction::Wierd(WierdScaledFunction::new(
                Arc::new(function.clone()),
                Arc::new(InternalNoise::new(
                    builtin_noise_params::SPAGHETTI_3D_1.clone(),
                    None,
                )),
                RarityMapper::Tunnels,
            ));

            let function4 = Arc::new(DensityFunction::Wierd(WierdScaledFunction::new(
                Arc::new(function),
                Arc::new(InternalNoise::new(
                    builtin_noise_params::SPAGHETTI_3D_2.clone(),
                    None,
                )),
                RarityMapper::Tunnels,
            )));

            let function5 = Arc::new(
                function3
                    .binary_max(function4)
                    .add(function2)
                    .clamp(-1f64, 1f64),
            );

            let function6 = CAVES_SPAGHETTI_ROUGHNESS_FUNCTION_OVERWORLD.clone();

            let function7 = DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    builtin_noise_params::CAVE_ENTRANCE.clone(),
                    None,
                )),
                0.75f64,
                0.5f64,
            ));

            let function8 = function7
                .add_const(0.37f64)
                .add(Arc::new(DensityFunction::ClampedY(YClampedFunction {
                    from: -10,
                    to: 30,
                    from_val: 0.3f64,
                    to_val: 0f64,
                })));

            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(function8.binary_min(Arc::new(function6.add(function5)))),
                WrapperType::CacheOnce,
            ))
        })
    });

    pub static CAVES_NOODLE_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            let function = Y.clone();

            let function2 = veritcal_range_choice(
                function.clone(),
                Arc::new(DensityFunction::Noise(NoiseFunction::new(
                    Arc::new(InternalNoise::new(
                        builtin_noise_params::NOODLE.clone(),
                        None,
                    )),
                    1f64,
                    1f64,
                ))),
                -60,
                320,
                -1,
            );

            let function3 = veritcal_range_choice(
                function.clone(),
                Arc::new(noise_in_range(
                    builtin_noise_params::NOODLE_THICKNESS.clone(),
                    1f64,
                    1f64,
                    -0.05f64,
                    -0.1f64,
                )),
                -60,
                320,
                0,
            );

            let function4 = veritcal_range_choice(
                function.clone(),
                Arc::new(DensityFunction::Noise(NoiseFunction::new(
                    Arc::new(InternalNoise::new(
                        builtin_noise_params::NOODLE_RIDGE_A.clone(),
                        None,
                    )),
                    2.6666666666666665f64,
                    2.6666666666666665f64,
                ))),
                -60,
                320,
                0,
            );

            let function5 = veritcal_range_choice(
                function.clone(),
                Arc::new(DensityFunction::Noise(NoiseFunction::new(
                    Arc::new(InternalNoise::new(
                        builtin_noise_params::NOODLE_RIDGE_B.clone(),
                        None,
                    )),
                    2.6666666666666665f64,
                    2.6666666666666665f64,
                ))),
                -60,
                320,
                0,
            );

            let function6 = Arc::new(
                function4
                    .abs()
                    .binary_max(Arc::new(function5.abs()))
                    .mul_const(1.5f64),
            );

            DensityFunction::Range(RangeFunction {
                input: Arc::new(function2),
                min: -1000000f64,
                max: 0f64,
                in_range: Arc::new(DensityFunction::Constant(ConstantFunction::new(64f64))),
                out_range: Arc::new(function3.add(function6)),
            })
        })
    });

    pub static CAVES_PILLARS_OVERWORLD: BuiltInNoise = LazyLock::new(|| {
        Arc::new({
            let function = DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    builtin_noise_params::PILLAR.clone(),
                    None,
                )),
                25f64,
                0.3f64,
            ));

            let function2 = Arc::new(noise_in_range(
                builtin_noise_params::PILLAR_RARENESS.clone(),
                1f64,
                1f64,
                0f64,
                -2f64,
            ));

            let function3 = noise_in_range(
                builtin_noise_params::PILLAR_THICKNESS.clone(),
                1f64,
                1f64,
                0f64,
                1.1f64,
            );

            let function4 = function.mul_const(2f64).add(function2);

            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(function4.mul(Arc::new(function3.cube()))),
                WrapperType::CacheOnce,
            ))
        })
    });

    fn sloped_cheese_function<'a>(
        jagged_noise: Arc<DensityFunction<'a>>,
        continents: Arc<DensityFunction<'a>>,
        erosion: Arc<DensityFunction<'a>>,
        ridges: Arc<DensityFunction<'a>>,
        ridges_folded: Arc<DensityFunction<'a>>,
        amplified: bool,
    ) -> SlopedCheeseResult<'a> {
        let offset = Arc::new(apply_blending(
            Arc::new(
                DensityFunction::Spline(SplineFunction::new(Arc::new(create_offset_spline(
                    continents.clone(),
                    erosion.clone(),
                    ridges.clone(),
                    amplified,
                ))))
                .add_const(-0.50375f32 as f64),
            ),
            BLEND_OFFSET.clone(),
        ));

        let factor = Arc::new(apply_blending(
            Arc::new(DensityFunction::Spline(SplineFunction::new(Arc::new(
                create_factor_spline(
                    continents.clone(),
                    erosion.clone(),
                    ridges.clone(),
                    ridges_folded.clone(),
                    amplified,
                ),
            )))),
            TEN.clone(),
        ));

        let depth = Arc::new(
            DensityFunction::ClampedY(YClampedFunction {
                from: -64,
                to: 320,
                from_val: 1.564,
                to_val: -1.5f64,
            })
            .add(offset.clone()),
        );

        let jaggedness = Arc::new(apply_blending(
            Arc::new(DensityFunction::Spline(SplineFunction::new(Arc::new(
                create_jaggedness_spline(continents, erosion, ridges, ridges_folded, amplified),
            )))),
            ZERO.clone(),
        ));

        let density1 = Arc::new(jaggedness.mul(Arc::new(jagged_noise.half_negative())));
        let density2 = DensityFunction::Constant(ConstantFunction::new(4f64)).mul(Arc::new(
            depth.add(density1).mul(factor.clone()).quarter_negative(),
        ));

        let sloped_cheese = Arc::new(density2.add(BASE_3D_NOISE_OVERWORLD.clone()));

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

pub fn veritcal_range_choice<'a>(
    input: Arc<DensityFunction<'a>>,
    in_range: Arc<DensityFunction<'a>>,
    min: i32,
    max: i32,
    out: i32,
) -> DensityFunction<'a> {
    DensityFunction::Wrapper(WrapperFunction::new(
        Arc::new(DensityFunction::Range(RangeFunction {
            input,
            min: min as f64,
            max: (max + 1) as f64,
            in_range,
            out_range: Arc::new(DensityFunction::Constant(ConstantFunction::new(out as f64))),
        })),
        WrapperType::Interpolated,
    ))
}

pub fn apply_blend_density(density: DensityFunction) -> DensityFunction {
    let function = DensityFunction::BlendDensity(BlendDensityFunction::new(Arc::new(density)));
    DensityFunction::Wrapper(WrapperFunction::new(
        Arc::new(function),
        WrapperType::Interpolated,
    ))
    .mul_const(0.64f64)
    .squeeze()
}

fn apply_blending<'a>(
    function: Arc<DensityFunction<'a>>,
    blend: Arc<DensityFunction<'a>>,
) -> DensityFunction<'a> {
    let function = lerp_density(built_in_noises::BLEND_ALPHA.clone(), blend, function);

    DensityFunction::Wrapper(WrapperFunction::new(
        Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
            Arc::new(function),
            WrapperType::Cache2D,
        ))),
        WrapperType::CacheFlat,
    ))
}

fn noise_in_range(
    noise: DoublePerlinNoiseParameters,
    xz_scale: f64,
    y_scale: f64,
    min: f64,
    max: f64,
) -> DensityFunction {
    map_range(
        Arc::new(DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(noise, None)),
            xz_scale,
            y_scale,
        ))),
        min,
        max,
    )
}

fn map_range(function: Arc<DensityFunction>, min: f64, max: f64) -> DensityFunction {
    let d = (min + max) * 0.5f64;
    let e = (max - min) * 0.5f64;

    DensityFunction::Constant(ConstantFunction::new(d)).add(Arc::new(
        DensityFunction::Constant(ConstantFunction::new(e)).mul(function),
    ))
}

#[derive(Clone)]
#[enum_dispatch(DensityFunctionImpl)]
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
    EndIsland(EndIslandFunction),
    Wierd(WierdScaledFunction<'a>),
    Range(RangeFunction<'a>),
    Wrapper(WrapperFunction<'a>),
    FlatCache(FlatCacheDensityFunction<'a>),
    Interpolator(InterpolatorDensityFunction<'a>),
}

impl<'a> DensityFunction<'a> {
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
        self.add(Arc::new(Self::Constant(ConstantFunction::new(val))))
    }

    pub fn add(&self, other: Arc<DensityFunction<'a>>) -> Self {
        BinaryFunction::create(BinaryType::Add, Arc::new(self.clone()), other)
    }

    pub fn mul_const(&self, val: f64) -> Self {
        self.mul(Arc::new(Self::Constant(ConstantFunction::new(val))))
    }

    pub fn mul(&self, other: Arc<DensityFunction<'a>>) -> Self {
        BinaryFunction::create(BinaryType::Mul, Arc::new(self.clone()), other)
    }

    pub fn binary_min(&self, other: Arc<DensityFunction<'a>>) -> Self {
        BinaryFunction::create(BinaryType::Min, Arc::new(self.clone()), other)
    }

    pub fn binary_max(&self, other: Arc<DensityFunction<'a>>) -> Self {
        BinaryFunction::create(BinaryType::Max, Arc::new(self.clone()), other)
    }
}

#[enum_dispatch(NoisePosImpl)]
pub enum NoisePos<'a> {
    Unblended(UnblendedNoisePos),
    ChunkNoise(ChunkNoiseSampler<'a>),
}

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
}

#[enum_dispatch]
pub trait NoisePosImpl {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn z(&self) -> i32;

    fn get_blender(&self) -> Blender {
        Blender::NoBlendBlender(NoBlendBlender {})
    }
}

#[enum_dispatch(ApplierImpl)]
pub enum Applier<'a> {
    ChunkNoise(ChunkNoiseSampler<'a>),
}

#[enum_dispatch]
pub trait ApplierImpl<'a> {
    fn at(&self, index: i32) -> NoisePos;

    fn fill(&self, densities: &[f64], function: &DensityFunction<'a>) -> Vec<f64>;
}

#[enum_dispatch(VisitorImpl)]
pub enum Visitor {
    Unwrap(UnwrapVisitor),
}

pub struct UnwrapVisitor {}

impl VisitorImpl for UnwrapVisitor {
    fn apply<'a>(&self, function: Arc<DensityFunction<'a>>) -> Arc<DensityFunction<'a>> {
        match function.deref() {
            DensityFunction::Wrapper(wrapper) => wrapper.wrapped(),
            _ => function.clone(),
        }
    }
}

#[enum_dispatch]
pub trait VisitorImpl {
    fn apply<'a>(&self, function: Arc<DensityFunction<'a>>) -> Arc<DensityFunction<'a>>;

    fn apply_internal_noise<'a>(&self, function: Arc<InternalNoise<'a>>) -> Arc<InternalNoise<'a>> {
        function.clone()
    }
}

#[enum_dispatch]
pub trait DensityFunctionImpl<'a> {
    fn sample(&self, pos: &NoisePos) -> f64;

    fn fill(&self, densities: &[f64], applier: &Applier) -> Vec<f64>;

    fn apply(&'a self, visitor: &'a Visitor) -> Arc<DensityFunction<'a>>;

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
    fn sample(&self, _pos: &NoisePos) -> f64 {
        self.value
    }

    fn fill(&self, densities: &[f64], _applier: &Applier) -> Vec<f64> {
        densities.iter().map(|_| self.value).collect()
    }

    fn apply(&'a self, visitor: &'a Visitor) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::Constant(self.clone())))
    }

    fn min(&self) -> f64 {
        self.value
    }

    fn max(&self) -> f64 {
        self.value
    }
}

#[derive(Clone)]
pub enum WrapperType {
    Cache2D,
    CacheFlat,
    CacheOnce,
    Interpolated,
}

#[derive(Clone)]
pub struct WrapperFunction<'a> {
    input: Arc<DensityFunction<'a>>,
    wrapper: WrapperType,
}

impl<'a> WrapperFunction<'a> {
    pub fn new(input: Arc<DensityFunction<'a>>, wrapper: WrapperType) -> Self {
        Self { input, wrapper }
    }

    pub fn wrapped(&self) -> Arc<DensityFunction<'a>> {
        self.input.clone()
    }
}

impl<'a> DensityFunctionImpl<'a> for WrapperFunction<'a> {
    fn max(&self) -> f64 {
        self.input.max()
    }

    fn min(&self) -> f64 {
        self.input.min()
    }

    fn sample(&self, pos: &NoisePos) -> f64 {
        self.input.sample(pos)
    }

    fn apply(&'a self, visitor: &'a Visitor) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::Wrapper(WrapperFunction {
            input: self.input.apply(visitor),
            wrapper: self.wrapper.clone(),
        })))
    }

    fn fill(&self, densities: &[f64], applier: &Applier) -> Vec<f64> {
        self.input.fill(densities, applier)
    }
}

#[derive(Clone)]
pub struct RangeFunction<'a> {
    input: Arc<DensityFunction<'a>>,
    min: f64,
    max: f64,
    in_range: Arc<DensityFunction<'a>>,
    out_range: Arc<DensityFunction<'a>>,
}

impl<'a> RangeFunction<'a> {
    pub fn new(
        input: Arc<DensityFunction<'a>>,
        min: f64,
        max: f64,
        in_range: Arc<DensityFunction<'a>>,
        out_range: Arc<DensityFunction<'a>>,
    ) -> Self {
        Self {
            input,
            min,
            max,
            in_range,
            out_range,
        }
    }
}

impl<'a> DensityFunctionImpl<'a> for RangeFunction<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        let d = self.input.sample(pos);
        if d >= self.min && d < self.max {
            self.in_range.sample(pos)
        } else {
            self.out_range.sample(pos)
        }
    }

    fn fill(&self, densities: &[f64], applier: &Applier) -> Vec<f64> {
        let densities = self.input.fill(densities, applier);
        densities
            .iter()
            .enumerate()
            .map(|(i, x)| {
                if *x >= self.min && *x < self.max {
                    self.in_range.sample(&applier.at(i as i32))
                } else {
                    self.out_range.sample(&applier.at(i as i32))
                }
            })
            .collect()
    }

    fn apply(&'a self, visitor: &'a Visitor) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::Range(RangeFunction {
            input: self.input.apply(visitor),
            min: self.min,
            max: self.max,
            in_range: self.in_range.apply(visitor),
            out_range: self.out_range.apply(visitor),
        })))
    }

    fn min(&self) -> f64 {
        self.in_range.min().min(self.out_range.min())
    }

    fn max(&self) -> f64 {
        self.in_range.max().max(self.out_range.max())
    }
}

#[derive(Clone)]
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

impl<'a> DensityFunctionImpl<'a> for YClampedFunction {
    fn sample(&self, pos: &NoisePos) -> f64 {
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

    fn fill(&self, densities: &[f64], applier: &Applier) -> Vec<f64> {
        applier.fill(densities, &DensityFunction::ClampedY(self.clone()))
    }

    fn apply(&'a self, visitor: &'a Visitor) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::ClampedY(self.clone())))
    }
}

pub trait UnaryDensityFunction<'a>: DensityFunctionImpl<'a> {
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
    delta: Arc<DensityFunction<'a>>,
    start: Arc<DensityFunction<'a>>,
    end: Arc<DensityFunction<'a>>,
) -> DensityFunction<'a> {
    if let DensityFunction::Constant(function) = start.as_ref() {
        lerp_density_static_start(delta, function.value, end)
    } else {
        let function = Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
            delta,
            WrapperType::CacheOnce,
        )));
        let function2 = Arc::new(function.mul_const(-1f64).add_const(1f64));
        start.mul(function2).add(Arc::new(end.mul(function)))
    }
}

pub fn lerp_density_static_start<'a>(
    delta: Arc<DensityFunction<'a>>,
    start: f64,
    end: Arc<DensityFunction<'a>>,
) -> DensityFunction<'a> {
    delta.mul(Arc::new(end.add_const(-start))).add_const(start)
}
