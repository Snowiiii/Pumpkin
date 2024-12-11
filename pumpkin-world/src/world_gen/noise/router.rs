use std::sync::{Arc, LazyLock};

use crate::world_gen::{
    noise::density::{
        apply_blend_density,
        basic::RangeFunction,
        built_in_density_function::{
            CAVES_ENTRANCES_OVERWORLD, CAVES_NOODLE_OVERWORLD, CONTINENTS_OVERWORLD,
            CONTINENTS_OVERWORLD_LARGE_BIOME, EROSION_OVERWORLD, EROSION_OVERWORLD_LARGE_BIOME,
            RIDGES_OVERWORLD, Y,
        },
        vertical_range_choice,
    },
    ore_sampler::vein_type,
};

use super::{
    built_in_noise_params,
    density::{
        basic::{ConstantFunction, WrapperFunction, WrapperType, YClampedFunction},
        built_in_density_function::{
            CAVES_PILLARS_OVERWORLD, CAVES_SPAGHETTI_2D_OVERWORLD,
            CAVES_SPAGHETTI_ROUGHNESS_FUNCTION_OVERWORLD, DEPTH_OVERWORLD,
            DEPTH_OVERWORLD_AMPLIFIED, DEPTH_OVERWORLD_LARGE_BIOME, FACTOR_OVERWORLD,
            FACTOR_OVERWORLD_AMPLIFIED, FACTOR_OVERWORLD_LARGE_BIOME, SHIFT_X, SHIFT_Z,
            SLOPED_CHEESE_OVERWORLD, SLOPED_CHEESE_OVERWORLD_AMPLIFIED,
            SLOPED_CHEESE_OVERWORLD_LARGE_BIOME, ZERO,
        },
        component_functions::{
            ComponentReference, ComponentReferenceMap, ComponentReferenceMath, ConverterImpl,
            DensityFunctionEnvironment, NoEnvironment, SharedComponentReference,
        },
        lerp_density_static_start,
        noise::{InternalNoise, NoiseFunction, ShiftedNoiseFunction},
    },
};

pub static OVERWORLD_NOISE_ROUTER: LazyLock<BaseRouter> =
    LazyLock::new(|| BaseRouter::create_surface_noise_router(false, false));
pub static OVERWORLD_NOISE_ROUTER_LARGE: LazyLock<BaseRouter> =
    LazyLock::new(|| BaseRouter::create_surface_noise_router(true, false));
pub static OVERWORLD_NOISE_ROUTER_AMPLIFIED: LazyLock<BaseRouter> =
    LazyLock::new(|| BaseRouter::create_surface_noise_router(false, true));

pub struct BaseRouter {
    pub(crate) barrier: SharedComponentReference,
    pub(crate) fluid_level_floodedness: SharedComponentReference,
    pub(crate) fluid_level_spread: SharedComponentReference,
    pub(crate) lava: SharedComponentReference,
    pub(crate) temperature: SharedComponentReference,
    pub(crate) vegetation: SharedComponentReference,
    pub(crate) continents: SharedComponentReference,
    pub(crate) erosion: SharedComponentReference,
    pub(crate) depth: SharedComponentReference,
    pub(crate) ridges: SharedComponentReference,
    pub(crate) internal_density: SharedComponentReference,
    pub(crate) final_density: SharedComponentReference,
    pub(crate) vein_toggle: SharedComponentReference,
    pub(crate) vein_ridged: SharedComponentReference,
    pub(crate) vein_gap: SharedComponentReference,
}

// TODO: This is double indirection... can we do something about that?
pub struct NoiseRouter<E: DensityFunctionEnvironment> {
    pub(crate) barrier: Box<dyn ComponentReference<E>>,
    pub(crate) fluid_level_floodedness: Box<dyn ComponentReference<E>>,
    pub(crate) fluid_level_spread: Box<dyn ComponentReference<E>>,
    pub(crate) lava: Box<dyn ComponentReference<E>>,
    pub(crate) temperature: Box<dyn ComponentReference<E>>,
    pub(crate) vegetation: Box<dyn ComponentReference<E>>,
    pub(crate) continents: Box<dyn ComponentReference<E>>,
    pub(crate) erosion: Box<dyn ComponentReference<E>>,
    pub(crate) depth: Box<dyn ComponentReference<E>>,
    pub(crate) ridges: Box<dyn ComponentReference<E>>,
    pub(crate) internal_density: Box<dyn ComponentReference<E>>,
    pub(crate) final_density: Box<dyn ComponentReference<E>>,
    pub(crate) vein_toggle: Box<dyn ComponentReference<E>>,
    pub(crate) vein_ridged: Box<dyn ComponentReference<E>>,
    pub(crate) vein_gap: Box<dyn ComponentReference<E>>,
}

impl BaseRouter {
    pub fn convert_assert_shared(
        &self,
        converter: &mut dyn ConverterImpl<NoEnvironment>,
    ) -> BaseRouter {
        BaseRouter {
            barrier: self.barrier.clone().convert(converter).assert_shared(),
            fluid_level_floodedness: self
                .fluid_level_floodedness
                .clone()
                .convert(converter)
                .assert_shared(),
            fluid_level_spread: self
                .fluid_level_spread
                .clone()
                .convert(converter)
                .assert_shared(),
            lava: self.lava.clone().convert(converter).assert_shared(),
            temperature: self.temperature.clone().convert(converter).assert_shared(),
            vegetation: self.vegetation.clone().convert(converter).assert_shared(),
            continents: self.continents.clone().convert(converter).assert_shared(),
            erosion: self.erosion.clone().convert(converter).assert_shared(),
            depth: self.depth.clone().convert(converter).assert_shared(),
            ridges: self.ridges.clone().convert(converter).assert_shared(),
            internal_density: self
                .internal_density
                .clone()
                .convert(converter)
                .assert_shared(),
            final_density: self
                .final_density
                .clone()
                .convert(converter)
                .assert_shared(),
            vein_toggle: self.vein_toggle.clone().convert(converter).assert_shared(),
            vein_ridged: self.vein_ridged.clone().convert(converter).assert_shared(),
            vein_gap: self.vein_gap.clone().convert(converter).assert_shared(),
        }
    }

    pub fn convert<E: DensityFunctionEnvironment>(
        &self,
        converter: &mut dyn ConverterImpl<E>,
    ) -> NoiseRouter<E> {
        NoiseRouter {
            barrier: self.barrier.clone().convert(converter).boxed(),
            fluid_level_floodedness: self
                .fluid_level_floodedness
                .clone()
                .convert(converter)
                .boxed(),
            fluid_level_spread: self.fluid_level_spread.clone().convert(converter).boxed(),
            lava: self.lava.clone().convert(converter).boxed(),
            temperature: self.temperature.clone().convert(converter).boxed(),
            vegetation: self.vegetation.clone().convert(converter).boxed(),
            continents: self.continents.clone().convert(converter).boxed(),
            erosion: self.erosion.clone().convert(converter).boxed(),
            depth: self.depth.clone().convert(converter).boxed(),
            ridges: self.ridges.clone().convert(converter).boxed(),
            internal_density: self.internal_density.clone().convert(converter).boxed(),
            final_density: self.final_density.clone().convert(converter).boxed(),
            vein_toggle: self.vein_toggle.clone().convert(converter).boxed(),
            vein_ridged: self.vein_ridged.clone().convert(converter).boxed(),
            vein_gap: self.vein_gap.clone().convert(converter).boxed(),
        }
    }
}

impl BaseRouter {
    pub fn create_surface_noise_router(large_biomes: bool, amplified: bool) -> Self {
        #[cfg(debug_assertions)]
        assert!(!(large_biomes && amplified));

        let aquifier_barrier = NoiseFunction::new(
            Arc::new(InternalNoise::new(
                &built_in_noise_params::AQUIFER_BARRIER,
                None,
            )),
            1f64,
            0.5f64,
        );

        let aquifier_fluid_level_floodedness = NoiseFunction::new(
            Arc::new(InternalNoise::new(
                &built_in_noise_params::AQUIFER_FLUID_LEVEL_FLOODEDNESS,
                None,
            )),
            1f64,
            0.67f64,
        );

        let aquifer_fluid_level_spread = NoiseFunction::new(
            Arc::new(InternalNoise::new(
                &built_in_noise_params::AQUIFER_FLUID_LEVEL_SPREAD,
                None,
            )),
            1f64,
            0.7142857142857143f64,
        );

        let aquifer_lava = NoiseFunction::new(
            Arc::new(InternalNoise::new(
                &built_in_noise_params::AQUIFER_LAVA,
                None,
            )),
            1f64,
            1f64,
        );

        let temperature = ShiftedNoiseFunction::<
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
                if large_biomes {
                    &built_in_noise_params::TEMPERATURE_LARGE
                } else {
                    &built_in_noise_params::TEMPERATURE
                },
                None,
            )),
        );

        let vegetation = ShiftedNoiseFunction::<
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
                if large_biomes {
                    &built_in_noise_params::VEGETATION_LARGE
                } else {
                    &built_in_noise_params::VEGETATION
                },
                None,
            )),
        );

        let factor_overworld = if large_biomes {
            FACTOR_OVERWORLD_LARGE_BIOME.clone()
        } else if amplified {
            FACTOR_OVERWORLD_AMPLIFIED.clone()
        } else {
            FACTOR_OVERWORLD.clone()
        };

        let depth_overworld = if large_biomes {
            DEPTH_OVERWORLD_LARGE_BIOME.clone()
        } else if amplified {
            DEPTH_OVERWORLD_AMPLIFIED.clone()
        } else {
            DEPTH_OVERWORLD.clone()
        };

        let mapped_depth_overworld = ConstantFunction::new(4f64).mul(
            depth_overworld
                .clone()
                .mul(
                    WrapperFunction::<NoEnvironment, SharedComponentReference>::new(
                        factor_overworld,
                        WrapperType::Cache2D,
                    )
                    .into(),
                )
                .quarter_negative(),
        );

        let sloped_cheese_overworld = if large_biomes {
            SLOPED_CHEESE_OVERWORLD_LARGE_BIOME.clone()
        } else if amplified {
            SLOPED_CHEESE_OVERWORLD_AMPLIFIED.clone()
        } else {
            SLOPED_CHEESE_OVERWORLD.clone()
        };

        let cave_entrances_overworld = sloped_cheese_overworld
            .clone()
            .min(ConstantFunction::new(5f64).mul(CAVES_ENTRANCES_OVERWORLD.clone()));

        let mapped_cave_entrances_overworld = RangeFunction::<
            NoEnvironment,
            SharedComponentReference,
            SharedComponentReference,
            SharedComponentReference,
        >::new(
            sloped_cheese_overworld.clone(),
            -1000000f64,
            1.5625f64,
            cave_entrances_overworld,
            create_caves(sloped_cheese_overworld),
        );

        let blended_cave_entrances_overworld = apply_blend_density(apply_surface_slides(
            amplified,
            mapped_cave_entrances_overworld.into(),
        ))
        .min(CAVES_NOODLE_OVERWORLD.clone());

        let i = vein_type::MIN_Y;
        let j = vein_type::MAX_Y;
        let ore_veininess = vertical_range_choice(
            Y.clone(),
            NoiseFunction::new(
                Arc::new(InternalNoise::new(
                    &built_in_noise_params::ORE_VEININESS,
                    None,
                )),
                1.5f64,
                1.5f64,
            )
            .into(),
            i,
            j,
            0,
        );

        let ore_vein_a = vertical_range_choice(
            Y.clone(),
            NoiseFunction::new(
                Arc::new(InternalNoise::new(&built_in_noise_params::ORE_VEIN_A, None)),
                4f64,
                4f64,
            )
            .into(),
            i,
            j,
            0,
        )
        .abs();

        let ore_vein_b = vertical_range_choice(
            Y.clone(),
            NoiseFunction::new(
                Arc::new(InternalNoise::new(&built_in_noise_params::ORE_VEIN_B, None)),
                4f64,
                4f64,
            )
            .into(),
            i,
            j,
            0,
        )
        .abs();

        let ore_vein = ConstantFunction::new(-0.08f32 as f64).add(ore_vein_a.max(ore_vein_b));

        let ore_gap = NoiseFunction::new(
            Arc::new(InternalNoise::new(&built_in_noise_params::ORE_GAP, None)),
            1f64,
            1f64,
        );

        Self {
            barrier: aquifier_barrier.into(),
            fluid_level_floodedness: aquifier_fluid_level_floodedness.into(),
            fluid_level_spread: aquifer_fluid_level_spread.into(),
            lava: aquifer_lava.into(),
            temperature: temperature.into(),
            vegetation: vegetation.into(),
            continents: if large_biomes {
                CONTINENTS_OVERWORLD_LARGE_BIOME.clone()
            } else {
                CONTINENTS_OVERWORLD.clone()
            },
            erosion: if large_biomes {
                EROSION_OVERWORLD_LARGE_BIOME.clone()
            } else {
                EROSION_OVERWORLD.clone()
            },
            depth: depth_overworld,
            ridges: RIDGES_OVERWORLD.clone(),
            internal_density: apply_surface_slides(
                amplified,
                mapped_depth_overworld
                    .add_const(-0.703125f64)
                    .clamp(-64f64, 64f64),
            ),
            final_density: blended_cave_entrances_overworld,
            vein_toggle: ore_veininess,
            vein_ridged: ore_vein,
            vein_gap: ore_gap.into(),
        }
    }
}

fn apply_surface_slides(
    amplified: bool,
    density: SharedComponentReference,
) -> SharedComponentReference {
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
    density: SharedComponentReference,
    y_min: i32,
    y_max: i32,
    top_rel_y_min: i32,
    top_rel_y_max: i32,
    top_density: f64,
    bottom_rel_y_min: i32,
    bottom_rel_y_max: i32,
    bottom_density: f64,
) -> SharedComponentReference {
    let function2 = YClampedFunction::new(
        y_min + y_max - top_rel_y_min,
        y_min + y_max - top_rel_y_max,
        1f64,
        0f64,
    );
    let function = lerp_density_static_start(function2.into(), top_density, density);
    let function3 = YClampedFunction::new(
        y_min + bottom_rel_y_min,
        y_min + bottom_rel_y_max,
        0f64,
        1f64,
    );
    lerp_density_static_start(function3.into(), bottom_density, function)
}

fn create_caves(sloped_cheese: SharedComponentReference) -> SharedComponentReference {
    let cave_layer = NoiseFunction::new(
        Arc::new(InternalNoise::new(&built_in_noise_params::CAVE_LAYER, None)),
        1f64,
        8f64,
    );
    let scaled_cave_layer = ConstantFunction::new(4f64).mul(cave_layer.square());
    let cave_cheese = NoiseFunction::new(
        Arc::new(InternalNoise::new(
            &built_in_noise_params::CAVE_CHEESE,
            None,
        )),
        1f64,
        0.6666666666666666f64,
    );
    let scaled_cave_cheese = ConstantFunction::new(0.27f64)
        .add(cave_cheese.into())
        .clamp(-1f64, 1f64)
        .add(
            ConstantFunction::new(1.5f64)
                .add(ConstantFunction::new(-0.64f64).mul(sloped_cheese))
                .clamp(0f64, 0.5f64),
        );
    let final_cave_layer = scaled_cave_layer.add(scaled_cave_cheese);
    let cave_entrances = final_cave_layer.min(CAVES_ENTRANCES_OVERWORLD.clone()).min(
        CAVES_SPAGHETTI_2D_OVERWORLD
            .clone()
            .add(CAVES_SPAGHETTI_ROUGHNESS_FUNCTION_OVERWORLD.clone()),
    );
    let scaled_pillars = RangeFunction::<
        NoEnvironment,
        SharedComponentReference,
        SharedComponentReference,
        SharedComponentReference,
    >::new(
        CAVES_PILLARS_OVERWORLD.clone(),
        -1000000f64,
        0.03f64,
        ConstantFunction::new(-1000000f64).into(),
        CAVES_PILLARS_OVERWORLD.clone(),
    );
    cave_entrances.max(scaled_pillars.into())
}

#[cfg(test)]
mod test {
    use std::{fs, path::Path, sync::Arc};

    use pumpkin_core::{
        assert_eq_delta,
        random::{legacy_rand::LegacyRand, xoroshiro128::Xoroshiro, RandomDeriver, RandomImpl},
    };

    use crate::{
        read_data_from_file,
        world_gen::noise::{
            config::LegacyChunkNoiseVisitor,
            density::{
                built_in_density_function::{EROSION_OVERWORLD, SLOPED_CHEESE_OVERWORLD},
                component_functions::{
                    ComponentReference, ComponentReferenceImplementation, ConversionResultPre,
                    ConverterEnvironment, ConverterImpl, NoEnvironment, OwnedConverterEnvironment,
                },
                noise::InternalNoise,
                NoisePos, UnblendedNoisePos,
            },
            perlin::DoublePerlinNoiseSampler,
            router::OVERWORLD_NOISE_ROUTER,
        },
    };

    use super::{apply_surface_slides, create_caves};

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
    fn test_apply_surface_slides() {
        let mut rand = LegacyRand::from_seed(0);
        let splitter = rand.next_splitter();
        let mut converter = TestConverter {
            splitter: RandomDeriver::Legacy(splitter),
        };
        let converted_func = EROSION_OVERWORLD
            .clone()
            .convert(&mut converter)
            .assert_shared();

        let values = [
            ((-1000, -1000), -0.30208879996359317f64),
            ((-1000, -800), -0.1076130122697041f64),
            ((-1000, -600), 0.05906607850421469f64),
            ((-1000, -400), 0.24726986724384714f64),
            ((-1000, -200), 0.2706794457383526f64),
            ((-1000, 0), 0.2697306747800193f64),
            ((-1000, 200), 0.21391910117663293f64),
            ((-1000, 400), 0.18535143299569468f64),
            ((-1000, 600), 0.1294413488921486f64),
            ((-1000, 800), 0.061580256900142794f64),
            ((-1000, 1000), 0.12530753721310528f64),
            ((-800, -1000), -0.26334816753858103f64),
            ((-800, -800), -0.020687744964915544f64),
            ((-800, -600), 0.023225727811309105f64),
            ((-800, -400), 0.07746776104719977f64),
            ((-800, -200), 0.09992470719841956f64),
            ((-800, 0), 0.1146551050894824f64),
            ((-800, 200), 0.06564485641223794f64),
            ((-800, 400), 0.08848851954273917f64),
            ((-800, 600), 0.056160953033021266f64),
            ((-800, 800), 0.011045253253628817f64),
            ((-800, 1000), 0.015198372445202502f64),
            ((-600, -1000), -0.16525687371490894f64),
            ((-600, -800), 0.05293810125583154f64),
            ((-600, -600), 0.17475689197213684f64),
            ((-600, -400), 0.16103184321844596f64),
            ((-600, -200), 0.04702586800266029f64),
            ((-600, 0), 0.008498755101605193f64),
            ((-600, 200), -0.05632912234656379f64),
            ((-600, 400), -0.03331984537443711f64),
            ((-600, 600), -0.15304483622814935f64),
            ((-600, 800), -0.10634904956802882f64),
            ((-600, 1000), -0.10145702807865031f64),
            ((-400, -1000), 0.014509904002985907f64),
            ((-400, -800), 0.1617721780377955f64),
            ((-400, -600), 0.1538795562844772f64),
            ((-400, -400), 0.09291099527948621f64),
            ((-400, -200), 0.016032945505656093f64),
            ((-400, 0), -0.12944528371858455f64),
            ((-400, 200), -0.22407726246974746f64),
            ((-400, 400), -0.18164279264712324f64),
            ((-400, 600), -0.15374226930846258f64),
            ((-400, 800), -0.14225372340399312f64),
            ((-400, 1000), -0.1707662008691604f64),
            ((-200, -1000), 0.1737449324031909f64),
            ((-200, -800), 0.15588767399915038f64),
            ((-200, -600), 0.09807388580637255f64),
            ((-200, -400), 0.04134584798831298f64),
            ((-200, -200), 0.07633793620513718f64),
            ((-200, 0), -0.16636855652213256f64),
            ((-200, 200), -0.13472373172620833f64),
            ((-200, 400), -0.22022531422260827f64),
            ((-200, 600), -0.1733817125721101f64),
            ((-200, 800), -0.25850640558184435f64),
            ((-200, 1000), -0.3016979165025152f64),
            ((0, -1000), 0.251814304135237f64),
            ((0, -800), 0.13332966941629143f64),
            ((0, -600), 0.10348476729023426f64),
            ((0, -400), 0.05536479756994106f64),
            ((0, -200), -0.01381233701693424f64),
            ((0, 0), -0.19098385460723655f64),
            ((0, 200), -0.2404237434218891f64),
            ((0, 400), -0.2610517555168873f64),
            ((0, 600), -0.2903199404423026f64),
            ((0, 800), -0.3374524245257239f64),
            ((0, 1000), -0.46305810978001316f64),
            ((200, -1000), 0.28709965377682267f64),
            ((200, -800), 0.19268673409560028f64),
            ((200, -600), 0.0041189472733734744f64),
            ((200, -400), -0.047481063682935865f64),
            ((200, -200), -0.17715255198738433f64),
            ((200, 0), -0.2656740328018877f64),
            ((200, 200), -0.38846940763264115f64),
            ((200, 400), -0.3771208707257109f64),
            ((200, 600), -0.42164320098305474f64),
            ((200, 800), -0.45957304404559196f64),
            ((200, 1000), -0.3886247688972352f64),
            ((400, -1000), 0.2946831456942509f64),
            ((400, -800), 0.15366391689596104f64),
            ((400, -600), -0.03683895527138398f64),
            ((400, -400), -0.12286969762482891f64),
            ((400, -200), -0.279692934897965f64),
            ((400, 0), -0.3299116026613176f64),
            ((400, 200), -0.4397636078519035f64),
            ((400, 400), -0.5016687095505773f64),
            ((400, 600), -0.4593088249355858f64),
            ((400, 800), -0.4120918663615397f64),
            ((400, 1000), -0.33349227727910835f64),
            ((600, -1000), 0.22507430288840974f64),
            ((600, -800), 0.11690723489828847f64),
            ((600, -600), -0.07736959302795038f64),
            ((600, -400), -0.3103436858126015f64),
            ((600, -200), -0.4646135074970098f64),
            ((600, 0), -0.42802179309528376f64),
            ((600, 200), -0.5031234101517712f64),
            ((600, 400), -0.43980110641169434f64),
            ((600, 600), -0.418150522213239f64),
            ((600, 800), -0.2858012217898377f64),
            ((600, 1000), -0.2078009597778817f64),
            ((800, -1000), 0.02646235055281687f64),
            ((800, -800), -0.0025711457048024355f64),
            ((800, -600), -0.22091967015314884f64),
            ((800, -400), -0.36062363978247114f64),
            ((800, -200), -0.5267644629903612f64),
            ((800, 0), -0.5221952728781099f64),
            ((800, 200), -0.5583524962468077f64),
            ((800, 400), -0.6064703416700034f64),
            ((800, 600), -0.42124877767264723f64),
            ((800, 800), -0.41023710661537083f64),
            ((800, 1000), -0.31011231029109043f64),
            ((1000, -1000), -0.13246158771459038f64),
            ((1000, -800), -0.21539115619228733f64),
            ((1000, -600), -0.2583347168279443f64),
            ((1000, -400), -0.3998592325985889f64),
            ((1000, -200), -0.5387601014248653f64),
            ((1000, 0), -0.5666190345839045f64),
            ((1000, 200), -0.6213722075094567f64),
            ((1000, 400), -0.671739773000314f64),
            ((1000, 600), -0.6238217558734026f64),
            ((1000, 800), -0.49441693548204757f64),
            ((1000, 1000), -0.3843933568068797f64),
        ];
        let amplified = apply_surface_slides(true, converted_func);
        for ((x, z), value) in values {
            let pos = &NoisePos::Unblended(UnblendedNoisePos::new(x, 60, z));
            assert_eq!(amplified.sample(pos), value);
        }
    }

    #[test]
    fn test_normal_surface_functions() {
        let pos = &NoisePos::Unblended(UnblendedNoisePos::new(0, 0, 0));
        assert_eq!(OVERWORLD_NOISE_ROUTER.barrier.sample(pos), 0f64);
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.fluid_level_floodedness.sample(pos),
            0f64
        );
        assert_eq!(OVERWORLD_NOISE_ROUTER.fluid_level_spread.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.lava.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.temperature.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.vegetation.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.continents.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.erosion.sample(pos), 0f64);
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.depth.sample(pos),
            0.3962499722838402f64
        );
        assert_eq!(OVERWORLD_NOISE_ROUTER.ridges.sample(pos), 0f64);
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.internal_density.sample(pos),
            8.093466727565826f64
        );
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.final_density.sample(pos),
            -0.07500000000000001f64
        );
        assert_eq!(OVERWORLD_NOISE_ROUTER.vein_toggle.sample(pos), 0f64);
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.vein_ridged.sample(pos),
            -0.07999999821186066f64
        );
        assert_eq!(OVERWORLD_NOISE_ROUTER.vein_gap.sample(pos), 0f64);

        let pos = &NoisePos::Unblended(UnblendedNoisePos::new(0, 60, 0));
        assert_eq!(OVERWORLD_NOISE_ROUTER.barrier.sample(pos), 0f64);
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.fluid_level_floodedness.sample(pos),
            0f64
        );
        assert_eq!(OVERWORLD_NOISE_ROUTER.fluid_level_spread.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.lava.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.temperature.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.vegetation.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.continents.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.erosion.sample(pos), 0f64);
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.depth.sample(pos),
            -0.07250002771615982f64
        );
        assert_eq!(OVERWORLD_NOISE_ROUTER.ridges.sample(pos), 0f64);
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.internal_density.sample(pos),
            -1.105492942375168f64
        );
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.final_density.sample(pos),
            -0.15607060320915436f64
        );
        assert_eq!(OVERWORLD_NOISE_ROUTER.vein_toggle.sample(pos), 0f64);
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.vein_ridged.sample(pos),
            -0.07999999821186066f64
        );
        assert_eq!(OVERWORLD_NOISE_ROUTER.vein_gap.sample(pos), 0f64);

        let pos = &NoisePos::Unblended(UnblendedNoisePos::new(0, 120, 0));
        assert_eq!(OVERWORLD_NOISE_ROUTER.barrier.sample(pos), 0f64);
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.fluid_level_floodedness.sample(pos),
            0f64
        );
        assert_eq!(OVERWORLD_NOISE_ROUTER.fluid_level_spread.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.lava.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.temperature.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.vegetation.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.continents.sample(pos), 0f64);
        assert_eq!(OVERWORLD_NOISE_ROUTER.erosion.sample(pos), 0f64);
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.depth.sample(pos),
            -0.5412500277161598f64
        );
        assert_eq!(OVERWORLD_NOISE_ROUTER.ridges.sample(pos), 0f64);
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.internal_density.sample(pos),
            -3.7070088166417925f64
        );
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.final_density.sample(pos),
            -0.4583333333333333f64
        );
        assert_eq!(OVERWORLD_NOISE_ROUTER.vein_toggle.sample(pos), 0f64);
        assert_eq!(
            OVERWORLD_NOISE_ROUTER.vein_ridged.sample(pos),
            -0.07999999821186066f64
        );
        assert_eq!(OVERWORLD_NOISE_ROUTER.vein_gap.sample(pos), 0f64);
    }

    #[test]
    fn test_converted_cave() {
        let mut rand = Xoroshiro::from_seed(0);
        let mut converter =
            LegacyChunkNoiseVisitor::new(RandomDeriver::Xoroshiro(rand.next_splitter()), 0);

        let expected_data: Vec<(i32, i32, i32, f64)> =
            read_data_from_file!("../../../assets/converted_cave_7_4.json");

        let function = create_caves(SLOPED_CHEESE_OVERWORLD.clone())
            .maybe_convert(&mut converter)
            .unwrap()
            .assert_shared();

        for (x, y, z, sample) in expected_data {
            let pos = NoisePos::Unblended(UnblendedNoisePos::new(x, y, z));
            assert_eq_delta!(function.sample(&pos), sample, f64::EPSILON);
        }
    }
}
