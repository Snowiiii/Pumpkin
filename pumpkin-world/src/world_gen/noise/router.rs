use std::sync::Arc;

use crate::world_gen::sampler::VeinType;

use super::{
    builtin_noise_params,
    density::{
        apply_blend_density, built_in_noises, lerp_density_static_start,
        noise::{InternalNoise, NoiseFunction, ShiftedNoiseFunction},
        veritcal_range_choice, ConstantFunction, DensityFunction, RangeFunction, Visitor,
        WrapperFunction, WrapperType, YClampedFunction,
    },
};

pub struct NoiseRouter<'a> {
    barrier: DensityFunction<'a>,
    fluid_level_floodedness: DensityFunction<'a>,
    fluid_level_spread: DensityFunction<'a>,
    lava: DensityFunction<'a>,
    temperature: DensityFunction<'a>,
    vegetation: DensityFunction<'a>,
    continents: DensityFunction<'a>,
    erosion: DensityFunction<'a>,
    depth: DensityFunction<'a>,
    ridges: DensityFunction<'a>,
    internal_density: DensityFunction<'a>,
    final_densitiy: DensityFunction<'a>,
    vein_toggle: DensityFunction<'a>,
    vein_ridged: DensityFunction<'a>,
    vein_gap: DensityFunction<'a>,
}

impl<'a> NoiseRouter<'a> {
    pub fn apply(&'a self, visitor: &'a impl Visitor) -> Self {
        NoiseRouter {
            barrier: self.barrier.apply(visitor),
            fluid_level_floodedness: self.fluid_level_floodedness.apply(visitor),
            fluid_level_spread: self.fluid_level_spread.apply(visitor),
            lava: self.lava.apply(visitor),
            temperature: self.temperature.apply(visitor),
            vegetation: self.vegetation.apply(visitor),
            continents: self.continents.apply(visitor),
            erosion: self.erosion.apply(visitor),
            depth: self.depth.apply(visitor),
            ridges: self.ridges.apply(visitor),
            internal_density: self.internal_density.apply(visitor),
            final_densitiy: self.final_densitiy.apply(visitor),
            vein_toggle: self.vein_toggle.apply(visitor),
            vein_ridged: self.vein_ridged.apply(visitor),
            vein_gap: self.vein_gap.apply(visitor),
        }
    }

    pub fn create_surface_noise_router(large_biomes: bool, amplified: bool) -> Self {
        let function = DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(
                builtin_noise_params::AQUIFER_BARRIER.clone(),
                None,
            )),
            1f64,
            0.5f64,
        ));

        let function2 = DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(
                builtin_noise_params::AQUIFER_BARRIER_FLOODEDNESS.clone(),
                None,
            )),
            1f64,
            0.67f64,
        ));

        let function3 = DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(
                builtin_noise_params::AQUIFER_FLUID_LEVEL_SPREAD.clone(),
                None,
            )),
            1f64,
            0.7142857142857143f64,
        ));

        let function4 = DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(
                builtin_noise_params::AQUIFER_LAVA.clone(),
                None,
            )),
            1f64,
            1f64,
        ));

        let function5 = built_in_noises::SHIFT_X.clone();
        let function6 = built_in_noises::SHIFT_Z.clone();

        let function7 = DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
            Arc::new(function5.clone()),
            Arc::new(built_in_noises::ZERO.clone()),
            Arc::new(function6.clone()),
            0.25f64,
            0f64,
            Arc::new(InternalNoise::new(
                if large_biomes {
                    builtin_noise_params::TEMPERATURE_LARGE.clone()
                } else {
                    builtin_noise_params::TEMPERATURE.clone()
                },
                None,
            )),
        ));

        let function8 = DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
            Arc::new(function5.clone()),
            Arc::new(built_in_noises::ZERO.clone()),
            Arc::new(function6.clone()),
            0.25f64,
            0f64,
            Arc::new(InternalNoise::new(
                if large_biomes {
                    builtin_noise_params::VEGETATION_LARGE.clone()
                } else {
                    builtin_noise_params::VEGETATION.clone()
                },
                None,
            )),
        ));

        let function9 = if large_biomes {
            built_in_noises::OVERWORLD_LARGE_SLOPED_CHEESE
                .factor
                .clone()
        } else if amplified {
            built_in_noises::OVERWORLD_AMPLIFIED_SLOPED_CHEESE
                .factor
                .clone()
        } else {
            built_in_noises::OVERWORLD_SLOPED_CHEESE.factor.clone()
        };

        let function10 = if large_biomes {
            built_in_noises::OVERWORLD_LARGE_SLOPED_CHEESE.depth.clone()
        } else if amplified {
            built_in_noises::OVERWORLD_AMPLIFIED_SLOPED_CHEESE
                .depth
                .clone()
        } else {
            built_in_noises::OVERWORLD_SLOPED_CHEESE.depth.clone()
        };

        let function11 = DensityFunction::Constant(ConstantFunction::new(4f64)).mul(
            function10
                .mul(DensityFunction::Wrapper(WrapperFunction::new(
                    Arc::new(function9),
                    WrapperType::Cache2D,
                )))
                .quarter_negative(),
        );

        let function12 = if large_biomes {
            built_in_noises::OVERWORLD_LARGE_SLOPED_CHEESE
                .sloped_cheese
                .clone()
        } else if amplified {
            built_in_noises::OVERWORLD_AMPLIFIED_SLOPED_CHEESE
                .sloped_cheese
                .clone()
        } else {
            built_in_noises::OVERWORLD_SLOPED_CHEESE
                .sloped_cheese
                .clone()
        };

        let function13 = function12.binary_min(
            DensityFunction::Constant(ConstantFunction::new(5f64))
                .mul(built_in_noises::CAVES_ENTRANCES_OVERWORLD.clone()),
        );

        let function14 = DensityFunction::Range(RangeFunction::new(
            Arc::new(function12.clone()),
            -1000000f64,
            1.5625f64,
            Arc::new(function13),
            Arc::new(create_caves(function12)),
        ));

        let function15 = apply_blend_density(apply_surface_slides(amplified, function14))
            .binary_min(built_in_noises::CAVES_NOODLE_OVERWORLD.clone());
        let function16 = built_in_noises::Y.clone();

        let i = VeinType::overall_min_y();
        let j = VeinType::overall_max_y();
        let function17 = veritcal_range_choice(
            function16.clone(),
            DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    builtin_noise_params::ORE_VEININESS.clone(),
                    None,
                )),
                1.5f64,
                1.5f64,
            )),
            i,
            j,
            0,
        );

        let function18 = veritcal_range_choice(
            function16.clone(),
            DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    builtin_noise_params::ORE_VEIN_A.clone(),
                    None,
                )),
                4f64,
                4f64,
            )),
            i,
            j,
            0,
        )
        .abs();

        let function19 = veritcal_range_choice(
            function16,
            DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    builtin_noise_params::ORE_VEIN_B.clone(),
                    None,
                )),
                4f64,
                4f64,
            )),
            i,
            j,
            0,
        )
        .abs();

        let function20 = DensityFunction::Constant(ConstantFunction::new(-0.08f64))
            .add(function18.binary_max(function19));

        let function21 = DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(
                builtin_noise_params::ORE_GAP.clone(),
                None,
            )),
            1f64,
            1f64,
        ));

        Self {
            barrier: function,
            fluid_level_floodedness: function2,
            fluid_level_spread: function3,
            lava: function4,
            temperature: function7,
            vegetation: function8,
            continents: if large_biomes {
                built_in_noises::CONTINENTS_OVERWORLD_LARGE_BIOME.clone()
            } else {
                built_in_noises::CONTINENTS_OVERWORLD.clone()
            },
            erosion: if large_biomes {
                built_in_noises::EROSION_OVERWORLD_LARGE_BIOME.clone()
            } else {
                built_in_noises::CONTINENTS_OVERWORLD.clone()
            },
            depth: function10,
            ridges: built_in_noises::RIDGES_OVERWORLD.clone(),
            internal_density: apply_surface_slides(
                amplified,
                function11.add_const(-0.703125).clamp(-64f64, 64f64),
            ),
            final_densitiy: function15,
            vein_toggle: function17,
            vein_ridged: function20,
            vein_gap: function21,
        }
    }
}

fn apply_surface_slides(amplified: bool, density: DensityFunction) -> DensityFunction {
    apply_slides(
        density,
        -64,
        384,
        if amplified { 16 } else { 80 },
        if amplified { 0 } else { 64 },
        -0.078125f64,
        0,
        24,
        if amplified { 0.4f64 } else { 0.1171875f64 },
    )
}

#[allow(clippy::too_many_arguments)]
fn apply_slides(
    density: DensityFunction,
    y_min: i32,
    y_max: i32,
    top_rel_y_min: i32,
    top_rel_y_max: i32,
    top_density: f64,
    bottom_rel_y_min: i32,
    bottom_rel_y_max: i32,
    bottom_density: f64,
) -> DensityFunction {
    let function2 = DensityFunction::ClampedY(YClampedFunction::new(
        y_min + y_max - top_rel_y_min,
        y_min + y_max + -top_rel_y_max,
        1f64,
        0f64,
    ));
    let function = lerp_density_static_start(function2, top_density, density);
    let function3 = DensityFunction::ClampedY(YClampedFunction::new(
        y_min + bottom_rel_y_min,
        y_min + bottom_rel_y_max,
        0f64,
        1f64,
    ));
    lerp_density_static_start(function3, bottom_density, function)
}

fn create_caves(sloped_cheese: DensityFunction) -> DensityFunction {
    let function = built_in_noises::CAVES_SPAGHETTI_2D_OVERWORLD.clone();
    let function2 = built_in_noises::CAVES_SPAGHETTI_ROUGHNESS_FUNCTION_OVERWORLD.clone();
    let function3 = DensityFunction::Noise(NoiseFunction::new(
        Arc::new(InternalNoise::new(
            builtin_noise_params::CAVE_LAYER.clone(),
            None,
        )),
        1f64,
        8f64,
    ));
    let function4 = DensityFunction::Constant(ConstantFunction::new(4f64)).mul(function3.square());
    let function5 = DensityFunction::Noise(NoiseFunction::new(
        Arc::new(InternalNoise::new(
            builtin_noise_params::CAVE_CHEESE.clone(),
            None,
        )),
        1f64,
        0.6666666666666666f64,
    ));
    let function6 = DensityFunction::Constant(ConstantFunction::new(0.27f64))
        .add(function5)
        .clamp(-1f64, 1f64)
        .add(
            DensityFunction::Constant(ConstantFunction::new(1.5f64))
                .add(DensityFunction::Constant(ConstantFunction::new(-0.64f64)).mul(sloped_cheese))
                .clamp(0f64, 0.5f64),
        );
    let function7 = function4.add(function6);
    let function8 = function7
        .binary_min(built_in_noises::CAVES_ENTRANCES_OVERWORLD.clone())
        .binary_min(function.add(function2));
    let function9 = built_in_noises::CAVES_PILLARS_OVERWORLD.clone();
    let function10 = DensityFunction::Range(RangeFunction::new(
        Arc::new(function9.clone()),
        -1000000f64,
        0.03f64,
        Arc::new(DensityFunction::Constant(ConstantFunction::new(
            -1000000f64,
        ))),
        Arc::new(function9),
    ));
    function8.binary_max(function10)
}
