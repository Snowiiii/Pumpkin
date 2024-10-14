use std::sync::Arc;

use crate::world_gen::sampler::VeinType;

use super::{
    density::{
        apply_blend_density, lerp_density_static_start,
        noise::{InternalNoise, NoiseFunction, ShiftedNoiseFunction},
        veritcal_range_choice, BuiltInNoiseFunctions, ConstantFunction, DensityFunction,
        DensityFunctionImpl, RangeFunction, Visitor, WrapperFunction, WrapperType,
        YClampedFunction,
    },
    BuiltInNoiseParams,
};

#[derive(Clone)]
pub struct NoiseRouter<'a> {
    barrier: Arc<DensityFunction<'a>>,
    fluid_level_floodedness: Arc<DensityFunction<'a>>,
    fluid_level_spread: Arc<DensityFunction<'a>>,
    lava: Arc<DensityFunction<'a>>,
    temperature: Arc<DensityFunction<'a>>,
    vegetation: Arc<DensityFunction<'a>>,
    continents: Arc<DensityFunction<'a>>,
    erosion: Arc<DensityFunction<'a>>,
    depth: Arc<DensityFunction<'a>>,
    ridges: Arc<DensityFunction<'a>>,
    pub(crate) internal_density: Arc<DensityFunction<'a>>,
    pub(crate) final_densitiy: Arc<DensityFunction<'a>>,
    vein_toggle: Arc<DensityFunction<'a>>,
    vein_ridged: Arc<DensityFunction<'a>>,
    vein_gap: Arc<DensityFunction<'a>>,
}

impl<'a> NoiseRouter<'a> {
    pub fn apply(&self, visitor: &Visitor<'a>) -> Self {
        Self {
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

    pub fn create_surface_noise_router(
        noise_params: &'a BuiltInNoiseParams<'a>,
        noise_funcs: &'a BuiltInNoiseFunctions<'a>,
        large_biomes: bool,
        amplified: bool,
    ) -> Self {
        let function = Arc::new(DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(
                noise_params.aquifer_barrier().clone(),
                None,
            )),
            1f64,
            0.5f64,
        )));

        let function2 = Arc::new(DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(
                noise_params.aquifer_fluid_level_floodedness().clone(),
                None,
            )),
            1f64,
            0.67f64,
        )));

        let function3 = Arc::new(DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(
                noise_params.aquifer_fluid_level_spread().clone(),
                None,
            )),
            1f64,
            0.7142857142857143f64,
        )));

        let function4 = Arc::new(DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(
                noise_params.aquifer_lava().clone(),
                None,
            )),
            1f64,
            1f64,
        )));

        let function5 = noise_funcs.shift_x().clone();
        let function6 = noise_funcs.shift_z().clone();

        let function7 = Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
            function5.clone(),
            noise_funcs.zero().clone(),
            function6.clone(),
            0.25f64,
            0f64,
            Arc::new(InternalNoise::new(
                if large_biomes {
                    noise_params.temperature_large().clone()
                } else {
                    noise_params.temperature().clone()
                },
                None,
            )),
        )));

        let function8 = Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
            function5.clone(),
            noise_funcs.zero().clone(),
            function6.clone(),
            0.25f64,
            0f64,
            Arc::new(InternalNoise::new(
                if large_biomes {
                    noise_params.vegetation_large().clone()
                } else {
                    noise_params.vegetation().clone()
                },
                None,
            )),
        )));

        let function9 = if large_biomes {
            noise_funcs.factor_overworld_large_biome().clone()
        } else if amplified {
            noise_funcs.factor_overworld_amplified().clone()
        } else {
            noise_funcs.factor_overworld().clone()
        };

        let function10 = if large_biomes {
            noise_funcs.depth_overworld_large_biome().clone()
        } else if amplified {
            noise_funcs.depth_overworld_amplified().clone()
        } else {
            noise_funcs.depth_overworld().clone()
        };

        let function11 = Arc::new(
            DensityFunction::Constant(ConstantFunction::new(4f64)).mul(Arc::new(
                function10
                    .mul(Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
                        function9,
                        WrapperType::Cache2D,
                    ))))
                    .quarter_negative(),
            )),
        );

        let function12 = if large_biomes {
            noise_funcs.sloped_cheese_overworld_large_biome().clone()
        } else if amplified {
            noise_funcs.sloped_cheese_overworld_amplified().clone()
        } else {
            noise_funcs.sloped_cheese_overworld().clone()
        };

        let function13 = Arc::new(
            function12.binary_min(Arc::new(
                DensityFunction::Constant(ConstantFunction::new(5f64))
                    .mul(noise_funcs.caves_entrances_overworld().clone()),
            )),
        );

        let function14 = Arc::new(DensityFunction::Range(RangeFunction::new(
            function12.clone(),
            -1000000f64,
            1.5625f64,
            function13,
            Arc::new(create_caves(noise_funcs, noise_params, function12)),
        )));

        let function15 = Arc::new(
            apply_blend_density(apply_surface_slides(amplified, function14))
                .binary_min(noise_funcs.caves_noodle_overworld().clone()),
        );
        let function16 = noise_funcs.y().clone();

        let i = VeinType::overall_min_y();
        let j = VeinType::overall_max_y();
        let function17 = Arc::new(veritcal_range_choice(
            function16.clone(),
            Arc::new(DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    noise_params.ore_veininess().clone(),
                    None,
                )),
                1.5f64,
                1.5f64,
            ))),
            i,
            j,
            0,
        ));

        let function18 = Arc::new(
            veritcal_range_choice(
                function16.clone(),
                Arc::new(DensityFunction::Noise(NoiseFunction::new(
                    Arc::new(InternalNoise::new(noise_params.ore_vein_a().clone(), None)),
                    4f64,
                    4f64,
                ))),
                i,
                j,
                0,
            )
            .abs(),
        );

        let function19 = Arc::new(
            veritcal_range_choice(
                function16,
                Arc::new(DensityFunction::Noise(NoiseFunction::new(
                    Arc::new(InternalNoise::new(noise_params.ore_vein_b().clone(), None)),
                    4f64,
                    4f64,
                ))),
                i,
                j,
                0,
            )
            .abs(),
        );

        let function20 = Arc::new(
            DensityFunction::Constant(ConstantFunction::new(-0.08f64))
                .add(Arc::new(function18.binary_max(function19))),
        );

        let function21 = Arc::new(DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(noise_params.ore_gap().clone(), None)),
            1f64,
            1f64,
        )));

        Self {
            barrier: function,
            fluid_level_floodedness: function2,
            fluid_level_spread: function3,
            lava: function4,
            temperature: function7,
            vegetation: function8,
            continents: if large_biomes {
                noise_funcs.continents_overworld_large_biome().clone()
            } else {
                noise_funcs.continents_overworld().clone()
            },
            erosion: if large_biomes {
                noise_funcs.erosion_overworld_large_biome().clone()
            } else {
                noise_funcs.erosion_overworld().clone()
            },
            depth: function10,
            ridges: noise_funcs.ridges_overworld().clone(),
            internal_density: Arc::new(apply_surface_slides(
                amplified,
                Arc::new(function11.add_const(-0.703125).clamp(-64f64, 64f64)),
            )),
            final_densitiy: function15,
            vein_toggle: function17,
            vein_ridged: function20,
            vein_gap: function21,
        }
    }
}

fn apply_surface_slides(amplified: bool, density: Arc<DensityFunction>) -> DensityFunction {
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
    density: Arc<DensityFunction>,
    y_min: i32,
    y_max: i32,
    top_rel_y_min: i32,
    top_rel_y_max: i32,
    top_density: f64,
    bottom_rel_y_min: i32,
    bottom_rel_y_max: i32,
    bottom_density: f64,
) -> DensityFunction {
    let function2 = Arc::new(DensityFunction::ClampedY(YClampedFunction::new(
        y_min + y_max - top_rel_y_min,
        y_min + y_max + -top_rel_y_max,
        1f64,
        0f64,
    )));
    let function = Arc::new(lerp_density_static_start(function2, top_density, density));
    let function3 = Arc::new(DensityFunction::ClampedY(YClampedFunction::new(
        y_min + bottom_rel_y_min,
        y_min + bottom_rel_y_max,
        0f64,
        1f64,
    )));
    lerp_density_static_start(function3, bottom_density, function)
}

fn create_caves<'a>(
    noise_funcs: &BuiltInNoiseFunctions<'a>,
    noise_params: &BuiltInNoiseParams<'a>,
    sloped_cheese: Arc<DensityFunction<'a>>,
) -> DensityFunction<'a> {
    let function = noise_funcs.caves_spaghetti_2d_overworld().clone();
    let function2 = noise_funcs
        .caves_spaghetti_roughness_function_overworld()
        .clone();
    let function3 = Arc::new(DensityFunction::Noise(NoiseFunction::new(
        Arc::new(InternalNoise::new(noise_params.cave_layer().clone(), None)),
        1f64,
        8f64,
    )));
    let function4 = Arc::new(
        DensityFunction::Constant(ConstantFunction::new(4f64)).mul(Arc::new(function3.square())),
    );
    let function5 = Arc::new(DensityFunction::Noise(NoiseFunction::new(
        Arc::new(InternalNoise::new(noise_params.cave_cheese().clone(), None)),
        1f64,
        0.6666666666666666f64,
    )));
    let function6 = Arc::new(
        DensityFunction::Constant(ConstantFunction::new(0.27f64))
            .add(function5)
            .clamp(-1f64, 1f64)
            .add(Arc::new(
                DensityFunction::Constant(ConstantFunction::new(1.5f64))
                    .add(Arc::new(
                        DensityFunction::Constant(ConstantFunction::new(-0.64f64))
                            .mul(sloped_cheese),
                    ))
                    .clamp(0f64, 0.5f64),
            )),
    );
    let function7 = Arc::new(function4.add(function6));
    let function8 = function7
        .binary_min(noise_funcs.caves_entrances_overworld().clone())
        .binary_min(Arc::new(function.add(function2)));
    let function9 = noise_funcs.caves_pillars_overworld().clone();
    let function10 = Arc::new(DensityFunction::Range(RangeFunction::new(
        function9.clone(),
        -1000000f64,
        0.03f64,
        Arc::new(DensityFunction::Constant(ConstantFunction::new(
            -1000000f64,
        ))),
        function9,
    )));
    function8.binary_max(function10)
}
