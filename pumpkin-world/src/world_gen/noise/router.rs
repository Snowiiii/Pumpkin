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
        let aquifier_barrier = Arc::new(DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(
                noise_params.aquifer_barrier().clone(),
                None,
            )),
            1f64,
            0.5f64,
        )));

        let aquifier_fluid_level_floodedness =
            Arc::new(DensityFunction::Noise(NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    noise_params.aquifer_fluid_level_floodedness().clone(),
                    None,
                )),
                1f64,
                0.67f64,
            )));

        let aquifer_fluid_level_spread = Arc::new(DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(
                noise_params.aquifer_fluid_level_spread().clone(),
                None,
            )),
            1f64,
            0.7142857142857143f64,
        )));

        let aquifer_lava = Arc::new(DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(
                noise_params.aquifer_lava().clone(),
                None,
            )),
            1f64,
            1f64,
        )));

        let shift_x = noise_funcs.shift_x().clone();
        let shift_z = noise_funcs.shift_z().clone();

        let temperature = Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
            shift_x.clone(),
            noise_funcs.zero().clone(),
            shift_z.clone(),
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

        let vegetation = Arc::new(DensityFunction::ShiftedNoise(ShiftedNoiseFunction::new(
            shift_x.clone(),
            noise_funcs.zero().clone(),
            shift_z.clone(),
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

        let factor_overworld = if large_biomes {
            noise_funcs.factor_overworld_large_biome().clone()
        } else if amplified {
            noise_funcs.factor_overworld_amplified().clone()
        } else {
            noise_funcs.factor_overworld().clone()
        };

        let depth_overworld = if large_biomes {
            noise_funcs.depth_overworld_large_biome().clone()
        } else if amplified {
            noise_funcs.depth_overworld_amplified().clone()
        } else {
            noise_funcs.depth_overworld().clone()
        };

        let mapped_depth_overworld = Arc::new(
            DensityFunction::Constant(ConstantFunction::new(4f64)).mul(Arc::new(
                depth_overworld
                    .mul(Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
                        factor_overworld,
                        WrapperType::Cache2D,
                    ))))
                    .quarter_negative(),
            )),
        );

        let sloped_cheese_overworld = if large_biomes {
            noise_funcs.sloped_cheese_overworld_large_biome().clone()
        } else if amplified {
            noise_funcs.sloped_cheese_overworld_amplified().clone()
        } else {
            noise_funcs.sloped_cheese_overworld().clone()
        };

        let cave_entrances_overworld = Arc::new(
            sloped_cheese_overworld.binary_min(Arc::new(
                DensityFunction::Constant(ConstantFunction::new(5f64))
                    .mul(noise_funcs.caves_entrances_overworld().clone()),
            )),
        );

        let mapped_cave_entraces_overworld = Arc::new(DensityFunction::Range(RangeFunction::new(
            sloped_cheese_overworld.clone(),
            -1000000f64,
            1.5625f64,
            cave_entrances_overworld,
            Arc::new(create_caves(
                noise_funcs,
                noise_params,
                sloped_cheese_overworld,
            )),
        )));

        let blended_cave_entrances_overworld = Arc::new(
            apply_blend_density(apply_surface_slides(
                amplified,
                mapped_cave_entraces_overworld,
            ))
            .binary_min(noise_funcs.caves_noodle_overworld().clone()),
        );
        let y = noise_funcs.y().clone();

        let i = VeinType::overall_min_y();
        let j = VeinType::overall_max_y();
        let ore_veininess = Arc::new(veritcal_range_choice(
            y.clone(),
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

        let ore_vein_a = Arc::new(
            veritcal_range_choice(
                y.clone(),
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

        let ore_vein_b = Arc::new(
            veritcal_range_choice(
                y,
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

        let ore_vein = Arc::new(
            DensityFunction::Constant(ConstantFunction::new(-0.08f64))
                .add(Arc::new(ore_vein_a.binary_max(ore_vein_b))),
        );

        let ore_gap = Arc::new(DensityFunction::Noise(NoiseFunction::new(
            Arc::new(InternalNoise::new(noise_params.ore_gap().clone(), None)),
            1f64,
            1f64,
        )));

        Self {
            barrier: aquifier_barrier,
            fluid_level_floodedness: aquifier_fluid_level_floodedness,
            fluid_level_spread: aquifer_fluid_level_spread,
            lava: aquifer_lava,
            temperature,
            vegetation,
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
            depth: depth_overworld,
            ridges: noise_funcs.ridges_overworld().clone(),
            internal_density: Arc::new(apply_surface_slides(
                amplified,
                Arc::new(
                    mapped_depth_overworld
                        .add_const(-0.703125)
                        .clamp(-64f64, 64f64),
                ),
            )),
            final_densitiy: blended_cave_entrances_overworld,
            vein_toggle: ore_veininess,
            vein_ridged: ore_vein,
            vein_gap: ore_gap,
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
    let caves_spaghetti_2d = noise_funcs.caves_spaghetti_2d_overworld().clone();
    let caves_spaghetti_roughness = noise_funcs
        .caves_spaghetti_roughness_function_overworld()
        .clone();
    let cave_layer = Arc::new(DensityFunction::Noise(NoiseFunction::new(
        Arc::new(InternalNoise::new(noise_params.cave_layer().clone(), None)),
        1f64,
        8f64,
    )));
    let scaled_cave_layer = Arc::new(
        DensityFunction::Constant(ConstantFunction::new(4f64)).mul(Arc::new(cave_layer.square())),
    );
    let cave_cheese = Arc::new(DensityFunction::Noise(NoiseFunction::new(
        Arc::new(InternalNoise::new(noise_params.cave_cheese().clone(), None)),
        1f64,
        0.6666666666666666f64,
    )));
    let scaled_cave_cheese = Arc::new(
        DensityFunction::Constant(ConstantFunction::new(0.27f64))
            .add(cave_cheese)
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
    let final_cave_layer = Arc::new(scaled_cave_layer.add(scaled_cave_cheese));
    let cave_entrances = final_cave_layer
        .binary_min(noise_funcs.caves_entrances_overworld().clone())
        .binary_min(Arc::new(caves_spaghetti_2d.add(caves_spaghetti_roughness)));
    let pillars = noise_funcs.caves_pillars_overworld().clone();
    let scaled_pillars = Arc::new(DensityFunction::Range(RangeFunction::new(
        pillars.clone(),
        -1000000f64,
        0.03f64,
        Arc::new(DensityFunction::Constant(ConstantFunction::new(
            -1000000f64,
        ))),
        pillars,
    )));
    cave_entrances.binary_max(scaled_pillars)
}
