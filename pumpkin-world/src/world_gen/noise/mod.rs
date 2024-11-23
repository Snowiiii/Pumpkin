use num_traits::Float;
pub mod config;
pub mod density;
pub mod perlin;
pub mod router;
mod simplex;

pub mod built_in_noise_params {
    use super::perlin::DoublePerlinNoiseParameters;

    pub const TEMPERATURE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -10,
        &[1.5f64, 0f64, 1f64, 0f64, 0f64, 0f64],
        "minecraft:temperature",
    );
    pub const VEGETATION: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -8,
        &[1f64, 1f64, 0f64, 0f64, 0f64, 0f64],
        "minecraft:vegetation",
    );
    pub const CONTINENTALNESS: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -9,
        &[1f64, 1f64, 2f64, 2f64, 2f64, 1f64, 1f64, 1f64, 1f64],
        "minecraft:continentalness",
    );
    pub const EROSION: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-9, &[1f64, 1f64, 0f64, 1f64, 1f64], "minecraft:erosion");
    pub const TEMPERATURE_LARGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -12,
        &[1.5f64, 0f64, 1f64, 0f64, 0f64, 0f64],
        "minecraft:temperature_large",
    );
    pub const VEGETATION_LARGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -10,
        &[1f64, 1f64, 0f64, 0f64, 0f64, 0f64],
        "minecraft:vegetation_large",
    );
    pub const CONTINENTALNESS_LARGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -11,
        &[1f64, 1f64, 2f64, 2f64, 2f64, 1f64, 1f64, 1f64, 1f64],
        "minecraft:continentalness_large",
    );
    pub const EROSION_LARGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -11,
        &[1f64, 1f64, 0f64, 1f64, 1f64],
        "minecraft:erosion_large",
    );
    pub const RIDGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -7,
        &[1f64, 2f64, 1f64, 0f64, 0f64, 0f64],
        "minecraft:ridge",
    );
    pub const OFFSET: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-3, &[1f64, 1f64, 1f64, 0f64], "minecraft:offset");
    pub const AQUIFER_BARRIER: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-3, &[1f64], "minecraft:aquifer_barrier");
    pub const AQUIFER_FLUID_LEVEL_FLOODEDNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7, &[1f64], "minecraft:aquifer_fluid_level_floodedness");
    pub const AQUIFER_LAVA: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-1, &[1f64], "minecraft:aquifer_lava");
    pub const AQUIFER_FLUID_LEVEL_SPREAD: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-5, &[1f64], "minecraft:aquifer_fluid_level_spread");
    pub const PILLAR: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7, &[1f64; 2], "minecraft:pillar");
    pub const PILLAR_RARENESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8, &[1f64], "minecraft:pillar_rareness");
    pub const PILLAR_THICKNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8, &[1f64], "minecraft:pillar_thickness");
    pub const SPAGHETTI_2D: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7, &[1f64], "minecraft:spaghetti_2d");
    pub const SPAGHETTI_2D_ELEVATION: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8, &[1f64], "minecraft:spaghetti_2d_elevation");
    pub const SPAGHETTI_2D_MODULATOR: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-11, &[1f64], "minecraft:spaghetti_2d_modulator");
    pub const SPAGHETTI_2D_THICKNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-11, &[1f64], "minecraft:spaghetti_2d_thickness");
    pub const SPAGHETTI_3D_1: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7, &[1f64], "minecraft:spaghetti_3d_1");
    pub const SPAGHETTI_3D_2: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7, &[1f64], "minecraft:spaghetti_3d_2");
    pub const SPAGHETTI_3D_RARITY: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-11, &[1f64], "minecraft:spaghetti_3d_rarity");
    pub const SPAGHETTI_3D_THICKNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8, &[1f64], "minecraft:spaghetti_3d_thickness");
    pub const SPAGHETTI_ROUGHNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-5, &[1f64], "minecraft:spaghetti_roughness");
    pub const SPAGHETTI_ROUGHNESS_MODULATOR: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8, &[1f64], "minecraft:spaghetti_roughness_modulator");
    pub const CAVE_ENTRANCE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7, &[0.4f64, 0.5f64, 1f64], "minecraft:cave_entrance");
    pub const CAVE_LAYER: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8, &[1f64], "minecraft:cave_layer");
    pub const CAVE_CHEESE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -8,
        &[0.5f64, 1f64, 2f64, 1f64, 2f64, 1f64, 0f64, 2f64, 0f64],
        "minecraft:cave_cheese",
    );
    pub const ORE_VEININESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8, &[1f64], "minecraft:ore_veininess");

    pub const ORE_VEIN_A: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7, &[1f64], "minecraft:ore_vein_a");
    pub const ORE_VEIN_B: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7, &[1f64], "minecraft:ore_vein_b");
    pub const ORE_GAP: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-5, &[1f64], "minecraft:ore_gap");
    pub const NOODLE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8, &[1f64], "minecraft:noodle");
    pub const NOODLE_THICKNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8, &[1f64], "minecraft:noodle_thickness");
    pub const NOODLE_RIDGE_A: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7, &[1f64], "minecraft:noodle_ridge_a");
    pub const NOODLE_RIDGE_B: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7, &[1f64], "minecraft:noodle_ridge_b");

    pub const JAGGED: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-16, &[1f64; 16], "minecraft:jagged");
    pub const SURFACE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-6, &[1f64; 3], "minecraft:surface");
    pub const SURFACE_SECONDARY: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -6,
        &[1f64, 1f64, 0f64, 1f64],
        "minecraft:surface_secondary",
    );
    pub const CLAY_BANDS_OFFSET: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8, &[1f64], "minecraft:clay_bands_offset");
    pub const BADLANDS_PILLAR: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-2, &[1f64; 4], "minecraft:badlands_pillar");
    pub const BADLANDS_PILLAR_ROOF: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8, &[1f64], "minecraft:badlands_pillar_roof");
    pub const BADLANDS_SURFACE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-6, &[1f64; 3], "minecraft:badlands_surface");
    pub const ICEBERG_PILLAR: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-6, &[1f64; 4], "minecraft:iceberg_pillar");
    pub const ICEBERG_PILLAR_ROOF: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-3, &[1f64], "minecraft:iceberg_pillar_roof");
    pub const ICEBERG_SURFACE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-6, &[1f64; 3], "minecraft:iceberg_surface");
    pub const SURFACE_SWAMP: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-2, &[1f64], "minecraft:surface_swamp");
    pub const CALCITE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-9, &[1f64; 4], "minecraft:calcite");
    pub const GRAVEL: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8, &[1f64; 4], "minecraft:gravel");
    pub const POWDER_SNOW: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-6, &[1f64; 4], "minecraft:powder_snow");
    pub const PACKED_ICE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7, &[1f64; 4], "minecraft:packed_ice");
    pub const ICE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-4, &[1f64; 4], "minecraft:ice");
    pub const SOUL_SAND_LAYER: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -8,
        &[
            1f64,
            1f64,
            1f64,
            1f64,
            0f64,
            0f64,
            0f64,
            0f64,
            0.013333333333333334f64,
        ],
        "minecraft:soul_sand_layer",
    );
    pub const GRAVEL_LAYER: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -8,
        &[
            1f64,
            1f64,
            1f64,
            1f64,
            0f64,
            0f64,
            0f64,
            0f64,
            0.013333333333333334f64,
        ],
        "minecraft:gravel_layer",
    );
    pub const PATCH: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -5,
        &[1f64, 0f64, 0f64, 0f64, 0f64, 0.013333333333333334f64],
        "minecraft:patch",
    );
    pub const NETHERRACK: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-3, &[1f64, 0f64, 0f64, 0.35f64], "minecraft:netherrack");
    pub const NETHER_WART: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-3, &[1f64, 0f64, 0f64, 0.9f64], "minecraft:nether_wart");
    pub const NETHER_STATE_SELECTOR: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-4, &[1f64], "minecraft:nether_state_selector");
}

pub fn lerp<T>(delta: T, start: T, end: T) -> T
where
    T: Float,
{
    start + delta * (end - start)
}

pub fn lerp_progress<T>(value: T, start: T, end: T) -> T
where
    T: Float,
{
    (value - start) / (end - start)
}

pub fn clamped_lerp(start: f64, end: f64, delta: f64) -> f64 {
    if delta < 0f64 {
        start
    } else if delta > 1f64 {
        end
    } else {
        lerp(delta, start, end)
    }
}

pub fn clamped_map(value: f64, old_start: f64, old_end: f64, new_start: f64, new_end: f64) -> f64 {
    clamped_lerp(new_start, new_end, lerp_progress(value, old_start, old_end))
}

pub fn map<T>(value: T, old_start: T, old_end: T, new_start: T, new_end: T) -> T
where
    T: Float,
{
    lerp(lerp_progress(value, old_start, old_end), new_start, new_end)
}

pub fn lerp2(delta_x: f64, delta_y: f64, x0y0: f64, x1y0: f64, x0y1: f64, x1y1: f64) -> f64 {
    lerp(
        delta_y,
        lerp(delta_x, x0y0, x1y0),
        lerp(delta_x, x0y1, x1y1),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn lerp3(
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    x0y0z0: f64,
    x1y0z0: f64,
    x0y1z0: f64,
    x1y1z0: f64,
    x0y0z1: f64,
    x1y0z1: f64,
    x0y1z1: f64,
    x1y1z1: f64,
) -> f64 {
    lerp(
        delta_z,
        lerp2(delta_x, delta_y, x0y0z0, x1y0z0, x0y1z0, x1y1z0),
        lerp2(delta_x, delta_y, x0y0z1, x1y0z1, x0y1z1, x1y1z1),
    )
}

struct Gradient {
    x: f64,
    y: f64,
    z: f64,
}

const GRADIENTS: [Gradient; 16] = [
    Gradient {
        x: 1f64,
        y: 1f64,
        z: 0f64,
    },
    Gradient {
        x: -1f64,
        y: 1f64,
        z: 0f64,
    },
    Gradient {
        x: 1f64,
        y: -1f64,
        z: 0f64,
    },
    Gradient {
        x: -1f64,
        y: -1f64,
        z: 0f64,
    },
    Gradient {
        x: 1f64,
        y: 0f64,
        z: 1f64,
    },
    Gradient {
        x: -1f64,
        y: 0f64,
        z: 1f64,
    },
    Gradient {
        x: 1f64,
        y: 0f64,
        z: -1f64,
    },
    Gradient {
        x: -1f64,
        y: 0f64,
        z: -1f64,
    },
    Gradient {
        x: 0f64,
        y: 1f64,
        z: 1f64,
    },
    Gradient {
        x: 0f64,
        y: -1f64,
        z: 1f64,
    },
    Gradient {
        x: 0f64,
        y: 1f64,
        z: -1f64,
    },
    Gradient {
        x: 0f64,
        y: -1f64,
        z: -1f64,
    },
    Gradient {
        x: 1f64,
        y: 1f64,
        z: 0f64,
    },
    Gradient {
        x: 0f64,
        y: -1f64,
        z: 1f64,
    },
    Gradient {
        x: -1f64,
        y: 1f64,
        z: 0f64,
    },
    Gradient {
        x: 0f64,
        y: -1f64,
        z: -1f64,
    },
];

impl Gradient {
    #[inline]
    fn dot(&self, x: f64, y: f64, z: f64) -> f64 {
        self.x * x + self.y * y + self.z * z
    }
}
