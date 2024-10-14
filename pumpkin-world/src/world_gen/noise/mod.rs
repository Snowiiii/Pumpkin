#![allow(dead_code)]

use derive_getters::Getters;
use perlin::DoublePerlinNoiseParameters;
pub mod chunk_sampler;
pub mod config;
pub mod density;
pub mod perlin;
mod router;
mod simplex;

#[derive(Getters)]
pub struct BuiltInNoiseParams<'a> {
    temperature: DoublePerlinNoiseParameters<'a>,
    vegetation: DoublePerlinNoiseParameters<'a>,
    continentalness: DoublePerlinNoiseParameters<'a>,
    erosion: DoublePerlinNoiseParameters<'a>,
    temperature_large: DoublePerlinNoiseParameters<'a>,
    vegetation_large: DoublePerlinNoiseParameters<'a>,
    continentalness_large: DoublePerlinNoiseParameters<'a>,
    erosion_large: DoublePerlinNoiseParameters<'a>,
    ridge: DoublePerlinNoiseParameters<'a>,
    offset: DoublePerlinNoiseParameters<'a>,
    aquifer_barrier: DoublePerlinNoiseParameters<'a>,
    aquifer_fluid_level_floodedness: DoublePerlinNoiseParameters<'a>,
    aquifer_lava: DoublePerlinNoiseParameters<'a>,
    aquifer_fluid_level_spread: DoublePerlinNoiseParameters<'a>,
    pillar: DoublePerlinNoiseParameters<'a>,
    pillar_rareness: DoublePerlinNoiseParameters<'a>,
    pillar_thickness: DoublePerlinNoiseParameters<'a>,
    spaghetti_2d: DoublePerlinNoiseParameters<'a>,
    spaghetti_2d_elevation: DoublePerlinNoiseParameters<'a>,
    spaghetti_2d_modulator: DoublePerlinNoiseParameters<'a>,
    spaghetti_2d_thickness: DoublePerlinNoiseParameters<'a>,
    spaghetti_3d_1: DoublePerlinNoiseParameters<'a>,
    spaghetti_3d_2: DoublePerlinNoiseParameters<'a>,
    spaghetti_3d_rarity: DoublePerlinNoiseParameters<'a>,
    spaghetti_3d_thickness: DoublePerlinNoiseParameters<'a>,
    spaghetti_roughness: DoublePerlinNoiseParameters<'a>,
    spaghetti_roughness_modulator: DoublePerlinNoiseParameters<'a>,
    cave_entrance: DoublePerlinNoiseParameters<'a>,
    cave_layer: DoublePerlinNoiseParameters<'a>,
    cave_cheese: DoublePerlinNoiseParameters<'a>,
    ore_veininess: DoublePerlinNoiseParameters<'a>,
    ore_vein_a: DoublePerlinNoiseParameters<'a>,
    ore_vein_b: DoublePerlinNoiseParameters<'a>,
    ore_gap: DoublePerlinNoiseParameters<'a>,
    noodle: DoublePerlinNoiseParameters<'a>,
    noodle_thickness: DoublePerlinNoiseParameters<'a>,
    noodle_ridge_a: DoublePerlinNoiseParameters<'a>,
    noodle_ridge_b: DoublePerlinNoiseParameters<'a>,
    jagged: DoublePerlinNoiseParameters<'a>,
    surface: DoublePerlinNoiseParameters<'a>,
    surface_secondary: DoublePerlinNoiseParameters<'a>,
    clay_bands_offset: DoublePerlinNoiseParameters<'a>,
    badlands_pillar: DoublePerlinNoiseParameters<'a>,
    badlands_pillar_roof: DoublePerlinNoiseParameters<'a>,
    badlands_surface: DoublePerlinNoiseParameters<'a>,
    iceberg_pillar: DoublePerlinNoiseParameters<'a>,
    iceberg_pillar_roof: DoublePerlinNoiseParameters<'a>,
    iceberg_surface: DoublePerlinNoiseParameters<'a>,
    surface_swamp: DoublePerlinNoiseParameters<'a>,
    calcite: DoublePerlinNoiseParameters<'a>,
    gravel: DoublePerlinNoiseParameters<'a>,
    powder_snow: DoublePerlinNoiseParameters<'a>,
    packed_ice: DoublePerlinNoiseParameters<'a>,
    ice: DoublePerlinNoiseParameters<'a>,
    soul_sand_layer: DoublePerlinNoiseParameters<'a>,
    gravel_layer: DoublePerlinNoiseParameters<'a>,
    patch: DoublePerlinNoiseParameters<'a>,
    netherrack: DoublePerlinNoiseParameters<'a>,
    nether_wart: DoublePerlinNoiseParameters<'a>,
    nether_state_selector: DoublePerlinNoiseParameters<'a>,
}

impl<'a> BuiltInNoiseParams<'a> {
    pub fn new() -> Self {
        Self {
            temperature: DoublePerlinNoiseParameters::new(
                -10,
                &[1.5f64, 0f64, 1f64, 0f64, 0f64, 0f64],
            ),
            vegetation: DoublePerlinNoiseParameters::new(-8, &[1f64, 1f64, 0f64, 0f64, 0f64, 0f64]),
            continentalness: DoublePerlinNoiseParameters::new(
                -9,
                &[1f64, 1f64, 2f64, 2f64, 2f64, 1f64, 1f64, 1f64, 1f64],
            ),

            erosion: DoublePerlinNoiseParameters::new(-9, &[1f64, 1f64, 0f64, 1f64, 1f64]),
            temperature_large: DoublePerlinNoiseParameters::new(
                -12,
                &[1.5f64, 0f64, 1f64, 0f64, 0f64, 0f64],
            ),
            vegetation_large: DoublePerlinNoiseParameters::new(
                -10,
                &[1f64, 1f64, 0f64, 0f64, 0f64, 0f64],
            ),
            continentalness_large: DoublePerlinNoiseParameters::new(
                -11,
                &[1f64, 1f64, 2f64, 2f64, 2f64, 1f64, 1f64, 1f64, 1f64],
            ),
            erosion_large: DoublePerlinNoiseParameters::new(-11, &[1f64, 1f64, 0f64, 1f64, 1f64]),
            ridge: DoublePerlinNoiseParameters::new(-7, &[1f64, 2f64, 1f64, 0f64, 0f64, 0f64]),
            offset: DoublePerlinNoiseParameters::new(-3, &[1f64; 4]),
            aquifer_barrier: DoublePerlinNoiseParameters::new(-3, &[1f64]),
            aquifer_fluid_level_floodedness: DoublePerlinNoiseParameters::new(-7, &[1f64]),
            aquifer_lava: DoublePerlinNoiseParameters::new(-1, &[1f64]),
            aquifer_fluid_level_spread: DoublePerlinNoiseParameters::new(-5, &[1f64]),
            pillar: DoublePerlinNoiseParameters::new(-7, &[1f64; 2]),
            pillar_rareness: DoublePerlinNoiseParameters::new(-8, &[1f64]),
            pillar_thickness: DoublePerlinNoiseParameters::new(-8, &[1f64]),
            spaghetti_2d: DoublePerlinNoiseParameters::new(-7, &[1f64]),
            spaghetti_2d_elevation: DoublePerlinNoiseParameters::new(-8, &[1f64]),
            spaghetti_2d_modulator: DoublePerlinNoiseParameters::new(-11, &[1f64]),
            spaghetti_2d_thickness: DoublePerlinNoiseParameters::new(-11, &[1f64]),
            spaghetti_3d_1: DoublePerlinNoiseParameters::new(-7, &[1f64]),
            spaghetti_3d_2: DoublePerlinNoiseParameters::new(-7, &[1f64]),
            spaghetti_3d_rarity: DoublePerlinNoiseParameters::new(-11, &[1f64]),
            spaghetti_3d_thickness: DoublePerlinNoiseParameters::new(-8, &[1f64]),
            spaghetti_roughness: DoublePerlinNoiseParameters::new(-5, &[1f64]),
            spaghetti_roughness_modulator: DoublePerlinNoiseParameters::new(-8, &[1f64]),
            cave_entrance: DoublePerlinNoiseParameters::new(-7, &[0.4f64, 0.5f64, 1f64]),
            cave_layer: DoublePerlinNoiseParameters::new(-8, &[1f64]),
            cave_cheese: DoublePerlinNoiseParameters::new(
                -8,
                &[0.5f64, 1f64, 2f64, 1f64, 2f64, 1f64, 0f64, 2f64, 0f64],
            ),
            ore_veininess: DoublePerlinNoiseParameters::new(-8, &[1f64]),
            ore_vein_a: DoublePerlinNoiseParameters::new(-7, &[1f64]),
            ore_vein_b: DoublePerlinNoiseParameters::new(-7, &[1f64]),
            ore_gap: DoublePerlinNoiseParameters::new(-5, &[1f64]),
            noodle: DoublePerlinNoiseParameters::new(-8, &[1f64]),
            noodle_thickness: DoublePerlinNoiseParameters::new(-8, &[1f64]),
            noodle_ridge_a: DoublePerlinNoiseParameters::new(-7, &[1f64]),
            noodle_ridge_b: DoublePerlinNoiseParameters::new(-7, &[1f64]),
            jagged: DoublePerlinNoiseParameters::new(-16, &[1f64; 16]),
            surface: DoublePerlinNoiseParameters::new(-6, &[1f64; 3]),
            surface_secondary: DoublePerlinNoiseParameters::new(-6, &[1f64, 1f64, 0f64, 1f64]),
            clay_bands_offset: DoublePerlinNoiseParameters::new(-8, &[1f64]),
            badlands_pillar: DoublePerlinNoiseParameters::new(-2, &[1f64; 4]),
            badlands_pillar_roof: DoublePerlinNoiseParameters::new(-8, &[1f64]),
            badlands_surface: DoublePerlinNoiseParameters::new(-6, &[1f64; 3]),
            iceberg_pillar: DoublePerlinNoiseParameters::new(-6, &[1f64; 4]),
            iceberg_pillar_roof: DoublePerlinNoiseParameters::new(-3, &[1f64]),
            iceberg_surface: DoublePerlinNoiseParameters::new(-6, &[1f64; 3]),
            surface_swamp: DoublePerlinNoiseParameters::new(-2, &[1f64]),
            calcite: DoublePerlinNoiseParameters::new(-9, &[1f64; 4]),
            gravel: DoublePerlinNoiseParameters::new(-8, &[1f64; 4]),
            powder_snow: DoublePerlinNoiseParameters::new(-6, &[1f64; 4]),
            packed_ice: DoublePerlinNoiseParameters::new(-7, &[1f64; 4]),
            ice: DoublePerlinNoiseParameters::new(-4, &[1f64; 4]),
            soul_sand_layer: DoublePerlinNoiseParameters::new(
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
            ),
            gravel_layer: DoublePerlinNoiseParameters::new(
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
            ),
            patch: DoublePerlinNoiseParameters::new(
                -5,
                &[1f64, 0f64, 0f64, 0f64, 0f64, 0.013333333333333334f64],
            ),
            netherrack: DoublePerlinNoiseParameters::new(-3, &[1f64, 0f64, 0f64, 0.35f64]),
            nether_wart: DoublePerlinNoiseParameters::new(-3, &[1f64, 0f64, 0f64, 0.9f64]),
            nether_state_selector: DoublePerlinNoiseParameters::new(-4, &[1f64]),
        }
    }
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
