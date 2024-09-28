use std::{ops::Deref, sync::Arc};

use blend::{BlendAlphaFunction, BlendDensityFunction, BlendOffsetFunction};
use derive_getters::Getters;
use end::EndIslandFunction;
use enum_dispatch::enum_dispatch;
use math::{BinaryFunction, BinaryType, LinearFunction};
use noise::{InternalNoise, InterpolatedNoiseSampler, NoiseFunction, ShiftedNoiseFunction};
use offset::{ShiftAFunction, ShiftBFunction};
use spline::SplineFunction;
use terrain_helpers::{create_factor_spline, create_jaggedness_spline, create_offset_spline};
use unary::{ClampFunction, UnaryFunction, UnaryType};
use weird::{RarityMapper, WierdScaledFunction};

use crate::world_gen::blender::Blender;

use super::{clamped_map, perlin::DoublePerlinNoiseParameters, BuiltInNoiseParams};

pub mod blend;
mod end;
mod math;
pub mod noise;
mod offset;
pub mod spline;
mod terrain_helpers;
mod unary;
mod weird;

struct SlopedCheeseResult<'a> {
    offset: Arc<DensityFunction<'a>>,
    factor: Arc<DensityFunction<'a>>,
    depth: Arc<DensityFunction<'a>>,
    jaggedness: Arc<DensityFunction<'a>>,
    sloped_cheese: Arc<DensityFunction<'a>>,
}

#[derive(Getters)]
pub struct BuiltInNoiseFunctions<'a> {
    zero: Arc<DensityFunction<'a>>,
    ten: Arc<DensityFunction<'a>>,
    blend_alpha: Arc<DensityFunction<'a>>,
    blend_offset: Arc<DensityFunction<'a>>,
    y: Arc<DensityFunction<'a>>,
    shift_x: Arc<DensityFunction<'a>>,
    shift_z: Arc<DensityFunction<'a>>,
    base_3d_noise_overworld: Arc<DensityFunction<'a>>,
    base_3d_noise_nether: Arc<DensityFunction<'a>>,
    base_3d_noise_end: Arc<DensityFunction<'a>>,
    continents_overworld: Arc<DensityFunction<'a>>,
    erosion_overworld: Arc<DensityFunction<'a>>,
    ridges_overworld: Arc<DensityFunction<'a>>,
    ridges_folded_overworld: Arc<DensityFunction<'a>>,
    offset_overworld: Arc<DensityFunction<'a>>,
    factor_overworld: Arc<DensityFunction<'a>>,
    jaggedness_overworld: Arc<DensityFunction<'a>>,
    depth_overworld: Arc<DensityFunction<'a>>,
    sloped_cheese_overworld: Arc<DensityFunction<'a>>,
    continents_overworld_large_biome: Arc<DensityFunction<'a>>,
    erosion_overworld_large_biome: Arc<DensityFunction<'a>>,
    offset_overworld_large_biome: Arc<DensityFunction<'a>>,
    factor_overworld_large_biome: Arc<DensityFunction<'a>>,
    jaggedness_overworld_large_biome: Arc<DensityFunction<'a>>,
    depth_overworld_large_biome: Arc<DensityFunction<'a>>,
    sloped_cheese_overworld_large_biome: Arc<DensityFunction<'a>>,
    offset_overworld_amplified: Arc<DensityFunction<'a>>,
    factor_overworld_amplified: Arc<DensityFunction<'a>>,
    jaggedness_overworld_amplified: Arc<DensityFunction<'a>>,
    depth_overworld_amplified: Arc<DensityFunction<'a>>,
    sloped_cheese_overworld_amplified: Arc<DensityFunction<'a>>,
    sloped_cheese_end: Arc<DensityFunction<'a>>,
    caves_spaghetti_roughness_function_overworld: Arc<DensityFunction<'a>>,
    caves_spaghetti_2d_thickness_modular_overworld: Arc<DensityFunction<'a>>,
    caves_spaghetti_2d_overworld: Arc<DensityFunction<'a>>,
    caves_entrances_overworld: Arc<DensityFunction<'a>>,
    caves_noodle_overworld: Arc<DensityFunction<'a>>,
    caves_pillars_overworld: Arc<DensityFunction<'a>>,
}

//Bits avaliable to encode y-pos
pub const SIZE_BITS_Y: i32 = 12;
pub const MAX_HEIGHT: i32 = (1 << SIZE_BITS_Y) - 32;
pub const MAX_COLUMN_HEIGHT: i32 = (MAX_HEIGHT >> 1) - 1;
pub const MIN_HEIGHT: i32 = MAX_COLUMN_HEIGHT - MAX_HEIGHT + 1;

impl<'a> BuiltInNoiseFunctions<'a> {
    pub fn new(built_in_noise_params: &BuiltInNoiseParams<'a>) -> Self {
        let blend_alpha = Arc::new(DensityFunction::BlendAlpha(BlendAlphaFunction {}));
        let blend_offset = Arc::new(DensityFunction::BlendOffset(BlendOffsetFunction {}));
        let zero = Arc::new(DensityFunction::Constant(ConstantFunction::new(0f64)));
        let ten = Arc::new(DensityFunction::Constant(ConstantFunction::new(10f64)));

        let y = Arc::new({
            DensityFunction::ClampedY(YClampedFunction {
                from: MIN_HEIGHT * 2,
                to: MAX_COLUMN_HEIGHT * 2,
                from_val: (MIN_HEIGHT * 2) as f64,
                to_val: (MAX_COLUMN_HEIGHT * 2) as f64,
            })
        });

        let shift_x = Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
                    Arc::new(DensityFunction::ShiftA(ShiftAFunction::new(Arc::new(
                        InternalNoise::new(built_in_noise_params.offset().clone(), None),
                    )))),
                    WrapperType::Cache2D,
                ))),
                WrapperType::CacheFlat,
            ))
        });

        let shift_z = Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
                    Arc::new(DensityFunction::ShiftB(ShiftBFunction::new(Arc::new(
                        InternalNoise::new(built_in_noise_params.offset().clone(), None),
                    )))),
                    WrapperType::Cache2D,
                ))),
                WrapperType::CacheFlat,
            ))
        });

        let base_3d_noise_overworld = Arc::new({
            DensityFunction::InterpolatedNoise(
                InterpolatedNoiseSampler::create_base_3d_noise_function(
                    0.25f64, 0.125f64, 80f64, 160f64, 8f64,
                ),
            )
        });

        let base_3d_noise_nether = Arc::new({
            DensityFunction::InterpolatedNoise(
                InterpolatedNoiseSampler::create_base_3d_noise_function(
                    0.25f64, 0.375f64, 80f64, 60f64, 8f64,
                ),
            )
        });

        let base_3d_noise_end = Arc::new({
            DensityFunction::InterpolatedNoise(
                InterpolatedNoiseSampler::create_base_3d_noise_function(
                    0.25f64, 0.25f64, 80f64, 160f64, 4f64,
                ),
            )
        });

        let continents_overworld = Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
                    shift_x.clone(),
                    zero.clone(),
                    shift_z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(
                        built_in_noise_params.continentalness().clone(),
                        None,
                    )),
                ))),
                WrapperType::CacheFlat,
            ))
        });

        let erosion_overworld = Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
                    shift_x.clone(),
                    zero.clone(),
                    shift_z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(
                        built_in_noise_params.erosion().clone(),
                        None,
                    )),
                ))),
                WrapperType::CacheFlat,
            ))
        });

        let ridges_overworld = Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
                    shift_x.clone(),
                    zero.clone(),
                    shift_z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(
                        built_in_noise_params.ridge().clone(),
                        None,
                    )),
                ))),
                WrapperType::CacheFlat,
            ))
        });

        let ridges_folded_overworld = Arc::new({
            ridges_overworld
                .abs()
                .add_const(-0.6666666666666666f64)
                .abs()
                .add_const(-0.3333333333333333f64)
                .mul_const(-3f64)
        });

        let overworld_sloped_cheese_result = sloped_cheese_function(
            Arc::new(DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    built_in_noise_params.jagged().clone(),
                    None,
                )),
                1500f64,
                0f64,
            ))),
            continents_overworld.clone(),
            erosion_overworld.clone(),
            ridges_overworld.clone(),
            ridges_folded_overworld.clone(),
            blend_offset.clone(),
            ten.clone(),
            zero.clone(),
            base_3d_noise_overworld.clone(),
            false,
        );

        let continents_overworld_large_biome = Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
                    shift_x.clone(),
                    zero.clone(),
                    shift_z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(
                        built_in_noise_params.continentalness_large().clone(),
                        None,
                    )),
                ))),
                WrapperType::CacheFlat,
            ))
        });

        let erosion_overworld_large_biome = Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
                    shift_x.clone(),
                    zero.clone(),
                    shift_z.clone(),
                    0.25f64,
                    0f64,
                    Arc::new(InternalNoise::new(
                        built_in_noise_params.erosion_large().clone(),
                        None,
                    )),
                ))),
                WrapperType::CacheFlat,
            ))
        });

        let overworld_large_biome_sloped_cheese_result = sloped_cheese_function(
            Arc::new(DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    built_in_noise_params.jagged().clone(),
                    None,
                )),
                1500f64,
                0f64,
            ))),
            continents_overworld_large_biome.clone(),
            erosion_overworld_large_biome.clone(),
            ridges_overworld.clone(),
            ridges_folded_overworld.clone(),
            blend_offset.clone(),
            ten.clone(),
            zero.clone(),
            base_3d_noise_overworld.clone(),
            false,
        );

        let overworld_amplified_sloped_cheese_result = sloped_cheese_function(
            Arc::new(DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    built_in_noise_params.jagged().clone(),
                    None,
                )),
                1500f64,
                0f64,
            ))),
            continents_overworld.clone(),
            erosion_overworld.clone(),
            ridges_overworld.clone(),
            ridges_folded_overworld.clone(),
            blend_offset.clone(),
            ten.clone(),
            zero.clone(),
            base_3d_noise_overworld.clone(),
            true,
        );

        let sloped_cheese_end = Arc::new({
            DensityFunction::EndIsland(EndIslandFunction::new(0)).add(base_3d_noise_end.clone())
        });

        let caves_spaghetti_roughness_function_overworld = Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(
                    noise_in_range(
                        built_in_noise_params
                            .spaghetti_roughness_modulator()
                            .clone(),
                        1f64,
                        1f64,
                        0f64,
                        -0.1f64,
                    )
                    .mul(Arc::new(
                        DensityFunction::Noise(NoiseFunction::new(
                            Arc::new(InternalNoise::new(
                                built_in_noise_params.spaghetti_roughness().clone(),
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
        });

        let caves_spaghetti_2d_thickness_modular_overworld = Arc::new({
            DensityFunction::Wrapper(WrapperFunction::new(
                Arc::new(noise_in_range(
                    built_in_noise_params.spaghetti_2d_thickness().clone(),
                    2f64,
                    1f64,
                    -0.6f64,
                    -1.3f64,
                )),
                WrapperType::CacheOnce,
            ))
        });

        let caves_spaghetti_2d_overworld = Arc::new({
            let function1 = DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    built_in_noise_params.spaghetti_2d_modulator().clone(),
                    None,
                )),
                2f64,
                1f64,
            ));

            let function2 = DensityFunction::Wierd(WierdScaledFunction::new(
                Arc::new(function1),
                Arc::new(InternalNoise::new(
                    built_in_noise_params.spaghetti_2d().clone(),
                    None,
                )),
                RarityMapper::Caves,
            ));

            let function3 = noise_in_range(
                built_in_noise_params.spaghetti_2d_elevation().clone(),
                1f64,
                0f64,
                ((-64i32) / 8i32) as f64,
                8f64,
            );

            let function4 = caves_spaghetti_2d_thickness_modular_overworld.clone();

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
        });

        let caves_entrances_overworld = Arc::new({
            let function = DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    built_in_noise_params.spaghetti_3d_rarity().clone(),
                    None,
                )),
                2f64,
                1f64,
            ));

            let function2 = Arc::new(noise_in_range(
                built_in_noise_params.spaghetti_3d_thickness().clone(),
                1f64,
                1f64,
                -0.065f64,
                -0.088f64,
            ));

            let function3 = DensityFunction::Wierd(WierdScaledFunction::new(
                Arc::new(function.clone()),
                Arc::new(InternalNoise::new(
                    built_in_noise_params.spaghetti_3d_1().clone(),
                    None,
                )),
                RarityMapper::Tunnels,
            ));

            let function4 = Arc::new(DensityFunction::Wierd(WierdScaledFunction::new(
                Arc::new(function),
                Arc::new(InternalNoise::new(
                    built_in_noise_params.spaghetti_3d_2().clone(),
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

            let function6 = caves_spaghetti_roughness_function_overworld.clone();

            let function7 = DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    built_in_noise_params.cave_entrance().clone(),
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
        });

        let caves_noodle_overworld = Arc::new({
            let function = y.clone();

            let function2 = veritcal_range_choice(
                function.clone(),
                Arc::new(DensityFunction::Noise(NoiseFunction::new(
                    Arc::new(InternalNoise::new(
                        built_in_noise_params.noodle().clone(),
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
                    built_in_noise_params.noodle_thickness().clone(),
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
                        built_in_noise_params.noodle_ridge_a().clone(),
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
                        built_in_noise_params.noodle_ridge_b().clone(),
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
        });

        let caves_pillars_overworld = Arc::new({
            let function = DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    built_in_noise_params.pillar().clone(),
                    None,
                )),
                25f64,
                0.3f64,
            ));

            let function2 = Arc::new(noise_in_range(
                built_in_noise_params.pillar_rareness().clone(),
                1f64,
                1f64,
                0f64,
                -2f64,
            ));

            let function3 = noise_in_range(
                built_in_noise_params.pillar_thickness().clone(),
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
        });

        Self {
            zero,
            ten,
            blend_offset,
            blend_alpha,
            y,
            shift_x,
            shift_z,
            base_3d_noise_overworld,
            base_3d_noise_nether,
            base_3d_noise_end,
            continents_overworld,
            erosion_overworld,
            ridges_overworld,
            ridges_folded_overworld,
            offset_overworld: overworld_sloped_cheese_result.offset,
            factor_overworld: overworld_sloped_cheese_result.factor,
            jaggedness_overworld: overworld_sloped_cheese_result.jaggedness,
            depth_overworld: overworld_sloped_cheese_result.depth,
            sloped_cheese_overworld: overworld_sloped_cheese_result.sloped_cheese,
            continents_overworld_large_biome,
            erosion_overworld_large_biome,
            offset_overworld_large_biome: overworld_large_biome_sloped_cheese_result.offset,
            factor_overworld_large_biome: overworld_large_biome_sloped_cheese_result.factor,
            jaggedness_overworld_large_biome: overworld_large_biome_sloped_cheese_result.jaggedness,
            depth_overworld_large_biome: overworld_large_biome_sloped_cheese_result.depth,
            sloped_cheese_overworld_large_biome: overworld_large_biome_sloped_cheese_result
                .sloped_cheese,
            offset_overworld_amplified: overworld_amplified_sloped_cheese_result.offset,
            factor_overworld_amplified: overworld_amplified_sloped_cheese_result.factor,
            jaggedness_overworld_amplified: overworld_amplified_sloped_cheese_result.jaggedness,
            depth_overworld_amplified: overworld_amplified_sloped_cheese_result.depth,
            sloped_cheese_overworld_amplified: overworld_amplified_sloped_cheese_result
                .sloped_cheese,
            sloped_cheese_end,
            caves_spaghetti_roughness_function_overworld,
            caves_spaghetti_2d_thickness_modular_overworld,
            caves_spaghetti_2d_overworld,
            caves_entrances_overworld,
            caves_noodle_overworld,
            caves_pillars_overworld,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn sloped_cheese_function<'a>(
    jagged_noise: Arc<DensityFunction<'a>>,
    continents: Arc<DensityFunction<'a>>,
    erosion: Arc<DensityFunction<'a>>,
    ridges: Arc<DensityFunction<'a>>,
    ridges_folded: Arc<DensityFunction<'a>>,
    blend_offset: Arc<DensityFunction<'a>>,
    ten: Arc<DensityFunction<'a>>,
    zero: Arc<DensityFunction<'a>>,
    base_3d_noise_overworld: Arc<DensityFunction<'a>>,
    amplified: bool,
) -> SlopedCheeseResult<'a> {
    let offset = Arc::new(apply_blending(
        Arc::new(
            DensityFunction::Constant(ConstantFunction::new(-0.50375f32 as f64)).add(Arc::new(
                DensityFunction::Spline(SplineFunction::new(Arc::new(create_offset_spline(
                    continents.clone(),
                    erosion.clone(),
                    ridges_folded.clone(),
                    amplified,
                )))),
            )),
        ),
        blend_offset,
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
        ten,
    ));

    let depth = Arc::new(
        DensityFunction::ClampedY(YClampedFunction {
            from: -64,
            to: 320,
            from_val: 1.5f64,
            to_val: -1.5f64,
        })
        .add(offset.clone()),
    );

    let jaggedness = Arc::new(apply_blending(
        Arc::new(DensityFunction::Spline(SplineFunction::new(Arc::new(
            create_jaggedness_spline(continents, erosion, ridges, ridges_folded, amplified),
        )))),
        zero,
    ));

    let density1 = Arc::new(jaggedness.mul(Arc::new(jagged_noise.half_negative())));
    let density2 = DensityFunction::Constant(ConstantFunction::new(4f64)).mul(Arc::new(
        depth.add(density1).mul(factor.clone()).quarter_negative(),
    ));

    let sloped_cheese = Arc::new(density2.add(base_3d_noise_overworld));

    SlopedCheeseResult {
        offset,
        factor,
        depth,
        jaggedness,
        sloped_cheese,
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
    let function = lerp_density(
        Arc::new(DensityFunction::BlendAlpha(BlendAlphaFunction {})),
        blend,
        function,
    );

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
}

impl<'a> DensityFunction<'a> {
    pub fn clamp(&self, min: f64, max: f64) -> Self {
        assert!(min <= max);
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

pub struct Unused<'a> {
    _x: &'a str,
}

impl<'a> NoisePosImpl for Unused<'a> {
    fn x(&self) -> i32 {
        todo!()
    }

    fn y(&self) -> i32 {
        todo!()
    }

    fn z(&self) -> i32 {
        todo!()
    }
}

impl<'a> ApplierImpl<'a> for Unused<'a> {
    fn at(&self, _index: usize) -> NoisePos<'a> {
        todo!()
    }

    fn fill(&self, _densities: &mut [f64], _function: &DensityFunction<'a>) {
        todo!()
    }
}

impl<'a> VisitorImpl<'a> for Unused<'a> {
    fn apply(&self, _function: Arc<DensityFunction<'a>>) -> Arc<DensityFunction<'a>> {
        todo!()
    }
}

#[enum_dispatch(NoisePosImpl)]
pub enum NoisePos<'a> {
    Unblended(UnblendedNoisePos),
    Todo(Unused<'a>),
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
        unimplemented!()
    }
}

#[enum_dispatch(ApplierImpl)]
pub enum Applier<'a> {
    Todo(Unused<'a>),
}

#[enum_dispatch]
pub trait ApplierImpl<'a> {
    fn at(&self, index: usize) -> NoisePos<'a>;

    fn fill(&self, densities: &mut [f64], function: &DensityFunction<'a>);
}

#[enum_dispatch(VisitorImpl)]
pub enum Visitor<'a> {
    Unwrap(UnwrapVisitor),
    Todo(Unused<'a>),
}

pub struct UnwrapVisitor {}

impl<'a> VisitorImpl<'a> for UnwrapVisitor {
    fn apply(&self, function: Arc<DensityFunction<'a>>) -> Arc<DensityFunction<'a>> {
        match function.deref() {
            DensityFunction::Wrapper(wrapper) => wrapper.wrapped(),
            _ => function.clone(),
        }
    }
}

#[enum_dispatch]
pub trait VisitorImpl<'a> {
    fn apply(&self, function: Arc<DensityFunction<'a>>) -> Arc<DensityFunction<'a>>;

    fn apply_internal_noise<'b>(&self, function: Arc<InternalNoise<'b>>) -> Arc<InternalNoise<'b>> {
        function.clone()
    }
}

#[enum_dispatch]
pub trait DensityFunctionImpl<'a> {
    fn sample(&self, pos: &NoisePos) -> f64;

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>);

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>>;

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

    fn fill(&self, densities: &mut [f64], _applier: &Applier) {
        densities.fill(self.value)
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
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
    CacheCell,
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

    pub fn wrapper(&self) -> WrapperType {
        self.wrapper.clone()
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

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::Wrapper(WrapperFunction {
            input: self.input.apply(visitor),
            wrapper: self.wrapper.clone(),
        })))
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
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

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
        self.input.fill(densities, applier);
        densities.iter_mut().enumerate().for_each(|(i, val)| {
            if *val >= self.min && *val < self.max {
                *val = self.in_range.sample(&applier.at(i));
            } else {
                *val = self.out_range.sample(&applier.at(i));
            }
        });
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
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

    fn fill(&self, densities: &mut [f64], applier: &Applier) {
        applier.fill(densities, &DensityFunction::ClampedY(self.clone()))
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
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

#[cfg(test)]
mod test {
    use crate::world_gen::noise::{density::DensityFunctionImpl, BuiltInNoiseParams};

    use super::{BuiltInNoiseFunctions, NoisePos, UnblendedNoisePos};

    #[test]
    fn test_density_function_correctness() {
        let noise_params = BuiltInNoiseParams::new();
        let noise_functions = BuiltInNoiseFunctions::new(&noise_params);

        let pos = NoisePos::Unblended(UnblendedNoisePos { x: 0, y: 0, z: 0 });

        assert_eq!(noise_functions.blend_alpha.sample(&pos), 1f64);
        assert_eq!(noise_functions.blend_alpha.min(), 1f64);
        assert_eq!(noise_functions.blend_alpha.max(), 1f64);

        assert_eq!(noise_functions.blend_offset.sample(&pos), 0f64);
        assert_eq!(noise_functions.blend_offset.min(), 0f64);
        assert_eq!(noise_functions.blend_offset.max(), 0f64);

        assert_eq!(noise_functions.zero.sample(&pos), 0f64);
        assert_eq!(noise_functions.zero.min(), 0f64);
        assert_eq!(noise_functions.zero.max(), 0f64);

        assert_eq!(noise_functions.y.sample(&pos), 0f64);
        assert_eq!(noise_functions.y.min(), -4064f64);
        assert_eq!(noise_functions.y.max(), 4062f64);

        assert_eq!(noise_functions.shift_x.sample(&pos), 0f64);
        assert_eq!(noise_functions.shift_x.min(), -8f64);
        assert_eq!(noise_functions.shift_x.max(), 8f64);

        assert_eq!(noise_functions.shift_z.sample(&pos), 0f64);
        assert_eq!(noise_functions.shift_z.min(), -8f64);
        assert_eq!(noise_functions.shift_z.max(), 8f64);

        assert_eq!(
            noise_functions.base_3d_noise_overworld.sample(&pos),
            0.05283727086562935f64
        );
        assert_eq!(
            noise_functions.base_3d_noise_overworld.min(),
            -87.55150000000002f64
        );
        assert_eq!(
            noise_functions.base_3d_noise_overworld.max(),
            87.55150000000002f64
        );

        assert_eq!(
            noise_functions.base_3d_noise_nether.sample(&pos),
            0.05283727086562935f64
        );
        assert_eq!(
            noise_functions.base_3d_noise_nether.min(),
            -258.65450000000004f64
        );
        assert_eq!(
            noise_functions.base_3d_noise_nether.max(),
            258.65450000000004f64
        );

        assert_eq!(
            noise_functions.base_3d_noise_end.sample(&pos),
            0.05283727086562935f64
        );
        assert_eq!(
            noise_functions.base_3d_noise_end.min(),
            -173.10299999999998f64
        );
        assert_eq!(
            noise_functions.base_3d_noise_end.max(),
            173.10299999999998f64
        );

        assert_eq!(noise_functions.continents_overworld.sample(&pos), 0f64);
        assert_eq!(noise_functions.continents_overworld.min(), -2f64);
        assert_eq!(noise_functions.continents_overworld.max(), 2f64);

        assert_eq!(noise_functions.erosion_overworld.sample(&pos), 0f64);
        assert_eq!(noise_functions.erosion_overworld.min(), -2f64);
        assert_eq!(noise_functions.erosion_overworld.max(), 2f64);

        assert_eq!(noise_functions.ridges_overworld.sample(&pos), 0f64);
        assert_eq!(noise_functions.ridges_overworld.min(), -2f64);
        assert_eq!(noise_functions.ridges_overworld.max(), 2f64);

        assert_eq!(noise_functions.ridges_folded_overworld.sample(&pos), -1f64);
        assert_eq!(
            noise_functions.ridges_folded_overworld.min(),
            -3.000000000000001f64
        );
        assert_eq!(noise_functions.ridges_folded_overworld.max(), 1f64);

        assert_eq!(
            noise_functions.offset_overworld.sample(&pos),
            -0.6037500277161598f64
        );
        assert_eq!(
            noise_functions.offset_overworld.min(),
            -1.3752707839012146f64
        );
        assert_eq!(
            noise_functions.offset_overworld.max(),
            0.9962499737739563f64
        );

        assert_eq!(
            noise_functions.factor_overworld.sample(&pos),
            5.549900531768799f64
        );
        assert_eq!(noise_functions.factor_overworld.min(), 0.625f64);
        assert_eq!(noise_functions.factor_overworld.max(), 6.300000190734863f64);

        assert_eq!(noise_functions.jaggedness_overworld.sample(&pos), 0f64);
        assert_eq!(noise_functions.jaggedness_overworld.min(), 0f64);
        assert_eq!(
            noise_functions.jaggedness_overworld.max(),
            0.6299999952316284f64
        );

        assert_eq!(
            noise_functions.depth_overworld.sample(&pos),
            0.3962499722838402f64
        );
        assert_eq!(
            noise_functions.depth_overworld.min(),
            -2.8752707839012146f64
        );
        assert_eq!(noise_functions.depth_overworld.max(), 2.4962499737739563f64);

        assert_eq!(
            noise_functions.sloped_cheese_overworld.sample(&pos),
            8.849428998431454f64
        );
        assert_eq!(
            noise_functions.sloped_cheese_overworld.min(),
            -109.63470657711427f64
        );
        assert_eq!(
            noise_functions.sloped_cheese_overworld.max(),
            182.2090019645691f64
        );

        assert_eq!(
            noise_functions
                .continents_overworld_large_biome
                .sample(&pos),
            0f64
        );
        assert_eq!(
            noise_functions.continents_overworld_large_biome.min(),
            -2f64
        );
        assert_eq!(noise_functions.continents_overworld_large_biome.max(), 2f64);

        assert_eq!(
            noise_functions.erosion_overworld_large_biome.sample(&pos),
            0f64
        );
        assert_eq!(noise_functions.erosion_overworld_large_biome.min(), -2f64);
        assert_eq!(noise_functions.erosion_overworld_large_biome.max(), 2f64);

        assert_eq!(
            noise_functions.offset_overworld_large_biome.sample(&pos),
            -0.6037500277161598f64
        );
        assert_eq!(
            noise_functions.offset_overworld_large_biome.min(),
            -1.3752707839012146f64
        );
        assert_eq!(
            noise_functions.offset_overworld_large_biome.max(),
            0.9962499737739563f64
        );

        assert_eq!(
            noise_functions.factor_overworld_large_biome.sample(&pos),
            5.549900531768799f64
        );
        assert_eq!(noise_functions.factor_overworld_large_biome.min(), 0.625f64);
        assert_eq!(
            noise_functions.factor_overworld_large_biome.max(),
            6.300000190734863f64
        );

        assert_eq!(
            noise_functions
                .jaggedness_overworld_large_biome
                .sample(&pos),
            0f64
        );
        assert_eq!(noise_functions.jaggedness_overworld_large_biome.min(), 0f64);
        assert_eq!(
            noise_functions.jaggedness_overworld_large_biome.max(),
            0.6299999952316284f64
        );

        assert_eq!(
            noise_functions.depth_overworld_large_biome.sample(&pos),
            0.3962499722838402f64
        );
        assert_eq!(
            noise_functions.depth_overworld_large_biome.min(),
            -2.8752707839012146f64
        );
        assert_eq!(
            noise_functions.depth_overworld_large_biome.max(),
            2.4962499737739563f64
        );

        assert_eq!(
            noise_functions
                .sloped_cheese_overworld_large_biome
                .sample(&pos),
            8.849428998431454f64
        );
        assert_eq!(
            noise_functions.sloped_cheese_overworld_large_biome.min(),
            -109.63470657711427f64
        );
        assert_eq!(
            noise_functions.sloped_cheese_overworld_large_biome.max(),
            182.2090019645691f64
        );

        assert_eq!(
            noise_functions.offset_overworld_amplified.sample(&pos),
            -0.6037500277161598f64
        );
        assert_eq!(
            noise_functions.offset_overworld_amplified.min(),
            -1.3752707839012146f64
        );
        assert_eq!(
            noise_functions.offset_overworld_amplified.max(),
            2.640259087085724f64
        );

        assert_eq!(
            noise_functions.factor_overworld_amplified.sample(&pos),
            0.6516130566596985f64
        );
        assert_eq!(
            noise_functions.factor_overworld_amplified.min(),
            0.13888883590698242f64
        );
        assert_eq!(
            noise_functions.factor_overworld_amplified.max(),
            6.300000190734863f64
        );

        assert_eq!(
            noise_functions.jaggedness_overworld_amplified.sample(&pos),
            0f64
        );
        assert_eq!(noise_functions.jaggedness_overworld_amplified.min(), 0f64);
        assert_eq!(
            noise_functions.jaggedness_overworld_amplified.max(),
            1.2599999904632568f64
        );

        assert_eq!(
            noise_functions.depth_overworld_amplified.sample(&pos),
            0.3962499722838402f64
        );
        assert_eq!(
            noise_functions.depth_overworld_amplified.min(),
            -2.8752707839012146f64
        );
        assert_eq!(
            noise_functions.depth_overworld_amplified.max(),
            4.140259087085724f64
        );

        assert_eq!(
            noise_functions
                .sloped_cheese_overworld_amplified
                .sample(&pos),
            1.085643893430405f64
        );
        assert_eq!(
            noise_functions.sloped_cheese_overworld_amplified.min(),
            -113.6037066672365f64
        );
        assert_eq!(
            noise_functions.sloped_cheese_overworld_amplified.max(),
            255.39003359528283f64
        );

        assert_eq!(
            noise_functions.sloped_cheese_end.sample(&pos),
            0.6153372708656294f64
        );
        assert_eq!(
            noise_functions.sloped_cheese_end.min(),
            -173.94674999999998f64
        );
        assert_eq!(
            noise_functions.sloped_cheese_end.max(),
            173.66549999999998f64
        );

        assert_eq!(
            noise_functions
                .caves_spaghetti_roughness_function_overworld
                .sample(&pos),
            0.020000000000000004f64
        );
        assert_eq!(
            noise_functions
                .caves_spaghetti_roughness_function_overworld
                .min(),
            -0.24000000000000005f64
        );
        assert_eq!(
            noise_functions
                .caves_spaghetti_roughness_function_overworld
                .max(),
            0.08000000000000002f64
        );

        assert_eq!(
            noise_functions.caves_entrances_overworld.sample(&pos),
            -0.056499999999999995f64
        );
        assert_eq!(noise_functions.caves_entrances_overworld.min(), -1.63f64);
        // NOTE: this doesn't match java but max/min is never used anywhere so
        assert_eq!(noise_functions.caves_entrances_overworld.max(), 1.08f64);

        assert_eq!(
            noise_functions.caves_noodle_overworld.sample(&pos),
            -0.07500000000000001f64
        );
        assert_eq!(noise_functions.caves_noodle_overworld.min(), -0.125f64);
        assert_eq!(noise_functions.caves_noodle_overworld.max(), 64f64);

        assert_eq!(noise_functions.caves_pillars_overworld.sample(&pos), -0.16637500000000005f64);
        assert_eq!(noise_functions.caves_pillars_overworld.min(), -31.44487500000001f64);
        assert_eq!(noise_functions.caves_pillars_overworld.max(), 22.460625000000007f64);

        assert_eq!(noise_functions.caves_spaghetti_2d_thickness_modular_overworld.sample(&pos), -0.95f64);
        assert_eq!(noise_functions.caves_spaghetti_2d_thickness_modular_overworld.min(), -1.65f64);
        assert_eq!(noise_functions.caves_spaghetti_2d_thickness_modular_overworld.max(), -0.2499999999999999f64);

        assert_eq!(noise_functions.caves_spaghetti_2d_overworld.sample(&pos), -0.07885f64);
        assert_eq!(noise_functions.caves_spaghetti_2d_overworld.min(), -1f64);
        assert_eq!(noise_functions.caves_spaghetti_2d_overworld.max(), 1f64);
    }
}
