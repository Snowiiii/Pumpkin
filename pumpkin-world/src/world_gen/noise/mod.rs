#![allow(dead_code)]
pub mod chunk_sampler;
pub mod config;
pub mod density;
pub mod perlin;
mod router;
mod simplex;

pub mod builtin_noise_params {
    use std::sync::LazyLock;

    use super::perlin::DoublePerlinNoiseParameters;

    pub static TEMPERATURE: LazyLock<DoublePerlinNoiseParameters<'static>> = LazyLock::new(|| {
        DoublePerlinNoiseParameters::new(-10, &[1.5f64, 0f64, 1f64, 0f64, 0f64, 0f64])
    });
    pub static VEGETATION: LazyLock<DoublePerlinNoiseParameters<'static>> = LazyLock::new(|| {
        DoublePerlinNoiseParameters::new(-8, &[1f64, 1f64, 0f64, 0f64, 0f64, 0f64])
    });
    pub static CONTINENTALNESS: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| {
            DoublePerlinNoiseParameters::new(
                -9,
                &[1f64, 1f64, 2f64, 2f64, 2f64, 1f64, 1f64, 1f64, 1f64],
            )
        });
    pub static EROSION: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-9, &[1f64, 1f64, 0f64, 1f64, 1f64]));
    pub static TEMPERATURE_LARGE: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| {
            DoublePerlinNoiseParameters::new(-12, &[1.5f64, 0f64, 1f64, 0f64, 0f64, 0f64])
        });
    pub static VEGETATION_LARGE: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| {
            DoublePerlinNoiseParameters::new(-10, &[1f64, 1f64, 0f64, 0f64, 0f64, 0f64])
        });
    pub static CONTINENTALNESS_LARGE: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| {
            DoublePerlinNoiseParameters::new(
                -11,
                &[1f64, 1f64, 2f64, 2f64, 2f64, 1f64, 1f64, 1f64, 1f64],
            )
        });
    pub static EROSION_LARGE: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-11, &[1f64, 1f64, 0f64, 1f64, 1f64]));
    pub static RIDGE: LazyLock<DoublePerlinNoiseParameters<'static>> = LazyLock::new(|| {
        DoublePerlinNoiseParameters::new(-7, &[1f64, 2f64, 1f64, 0f64, 0f64, 0f64])
    });
    pub static OFFSET: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-3, &[1f64; 4]));
    pub static AQUIFER_BARRIER: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-3, &[1f64]));
    pub static AQUIFER_BARRIER_FLOODEDNESS: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-7, &[1f64]));
    pub static AQUIFER_LAVA: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-1, &[1f64]));
    pub static AQUIFER_FLUID_LEVEL_SPREAD: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-5, &[1f64]));
    pub static PILLAR: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-7, &[1f64; 2]));
    pub static PILLAR_RARENESS: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-8, &[1f64]));
    pub static PILLAR_THICKNESS: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-8, &[1f64]));
    pub static SPAGHETTI_2D: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-7, &[1f64]));
    pub static SPAGHETTI_2D_ELEVATION: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-8, &[1f64]));
    pub static SPAGHETTI_2D_MODULATOR: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-11, &[1f64]));
    pub static SPAGHETTI_2D_THICKNESS: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-11, &[1f64]));
    pub static SPAGHETTI_3D_1: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-7, &[1f64]));
    pub static SPAGHETTI_3D_2: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-7, &[1f64]));
    pub static SPAGHETTI_3D_RARITY: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-11, &[1f64]));
    pub static SPAGHETTI_3D_THICKNESS: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-8, &[1f64]));
    pub static SPAGHETTI_ROUGHNESS: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-5, &[1f64]));
    pub static SPAGHETTI_ROUGHNESS_MODULATOR: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-8, &[1f64]));
    pub static CAVE_ENTRANCE: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-7, &[0.4f64, 0.5f64, 1f64]));
    pub static CAVE_LAYER: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-8, &[1f64]));
    pub static CAVE_CHEESE: LazyLock<DoublePerlinNoiseParameters<'static>> = LazyLock::new(|| {
        DoublePerlinNoiseParameters::new(
            -8,
            &[0.5f64, 1f64, 2f64, 1f64, 2f64, 1f64, 0f64, 2f64, 0f64],
        )
    });
    pub static ORE_VEININESS: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-8, &[1f64]));
    pub static ORE_VEIN_A: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-7, &[1f64]));
    pub static ORE_VEIN_B: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-7, &[1f64]));
    pub static ORE_GAP: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-5, &[1f64]));
    pub static NOODLE: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-8, &[1f64]));
    pub static NOODLE_THICKNESS: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-8, &[1f64]));
    pub static NOODLE_RIDGE_A: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-7, &[1f64]));
    pub static NOODLE_RIDGE_B: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-7, &[1f64]));
    pub static JAGGED: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-16, &[1f64; 16]));
    pub static SURFACE: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-6, &[1f64; 3]));
    pub static SURFACE_SECONDARY: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-6, &[1f64, 1f64, 0f64, 1f64]));
    pub static CLAY_BANDS_OFFSET: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-8, &[1f64]));
    pub static BADLANDS_PILLAR: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-2, &[1f64; 4]));
    pub static BADLANDS_PILLAR_ROOF: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-8, &[1f64]));
    pub static BADLANDS_SURFACE: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-6, &[1f64; 3]));
    pub static ICEBERG_PILLAR: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-6, &[1f64; 4]));
    pub static ICEBERG_PILLAR_ROOF: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-3, &[1f64]));
    pub static ICEBERG_SURFACE: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-6, &[1f64; 3]));
    pub static SURFACE_SWAMP: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-2, &[1f64]));
    pub static CALCITE: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-9, &[1f64; 4]));
    pub static GRAVEL: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-8, &[1f64; 4]));
    pub static POWDER_SNOW: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-6, &[1f64; 4]));
    pub static PACKED_ICE: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-7, &[1f64; 4]));
    pub static ICE: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-4, &[1f64; 4]));
    pub static SOUL_SAND_LAYER: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| {
            DoublePerlinNoiseParameters::new(
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
            )
        });
    pub static GRAVEL_LAYER: LazyLock<DoublePerlinNoiseParameters<'static>> = LazyLock::new(|| {
        DoublePerlinNoiseParameters::new(
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
        )
    });
    pub static PATCH: LazyLock<DoublePerlinNoiseParameters<'static>> = LazyLock::new(|| {
        DoublePerlinNoiseParameters::new(
            -5,
            &[1f64, 0f64, 0f64, 0f64, 0f64, 0.013333333333333334f64],
        )
    });
    pub static NETHERRACK: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-3, &[1f64, 0f64, 0f64, 0.35f64]));
    pub static NETHER_WART: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-3, &[1f64, 0f64, 0f64, 0.9f64]));
    pub static NETHER_STATE_SELECTOR: LazyLock<DoublePerlinNoiseParameters<'static>> =
        LazyLock::new(|| DoublePerlinNoiseParameters::new(-4, &[1f64]));
}

pub fn lerp_32(delta: f32, start: f32, end: f32) -> f32 {
    start + delta * (end - start)
}

pub fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
}

pub fn lerp_progress(value: f64, start: f64, end: f64) -> f64 {
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
    x: i32,
    y: i32,
    z: i32,
}

const GRADIENTS: [Gradient; 16] = [
    Gradient { x: 1, y: 1, z: 0 },
    Gradient { x: -1, y: 1, z: 0 },
    Gradient { x: 1, y: -1, z: 0 },
    Gradient { x: -1, y: -1, z: 0 },
    Gradient { x: 1, y: 0, z: 1 },
    Gradient { x: -1, y: 0, z: 1 },
    Gradient { x: 1, y: 0, z: -1 },
    Gradient { x: -1, y: 0, z: -1 },
    Gradient { x: 0, y: 1, z: 1 },
    Gradient { x: 0, y: -1, z: 1 },
    Gradient { x: 0, y: 1, z: -1 },
    Gradient { x: 0, y: -1, z: -1 },
    Gradient { x: 1, y: 1, z: 0 },
    Gradient { x: 0, y: -1, z: 1 },
    Gradient { x: -1, y: 1, z: 0 },
    Gradient { x: 0, y: -1, z: -1 },
];

fn dot(gradient: &Gradient, x: f64, y: f64, z: f64) -> f64 {
    gradient.x as f64 * x + gradient.y as f64 * y + gradient.z as f64 * z
}
