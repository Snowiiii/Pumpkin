#![allow(dead_code)]
mod density;
mod perlin;
mod simplex;

pub mod builtin_noise_params {
    use lazy_static::lazy_static;

    use super::perlin::DoublePerlinNoiseParameters;
    lazy_static! {
        pub static ref temperature: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-10, &[1.5f64, 0f64, 1f64, 0f64, 0f64, 0f64,],);
        pub static ref vegetation: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-8, &[1f64, 1f64, 0f64, 0f64, 0f64, 0f64,],);
        pub static ref continentalness: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(
                -9,
                &[1f64, 1f64, 2f64, 2f64, 2f64, 1f64, 1f64, 1f64, 1f64,],
            );
        pub static ref erosion: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-9, &[1f64, 1f64, 0f64, 1f64, 1f64,],);
        pub static ref temperature_large: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-12, &[1.5f64, 0f64, 1f64, 0f64, 0f64, 0f64,],);
        pub static ref vegetation_large: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-10, &[1f64, 1f64, 0f64, 0f64, 0f64, 0f64,],);
        pub static ref continentalness_large: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(
                -11,
                &[1f64, 1f64, 2f64, 2f64, 2f64, 1f64, 1f64, 1f64, 1f64,],
            );
        pub static ref erosion_large: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-11, &[1f64, 1f64, 0f64, 1f64, 1f64,],);
        pub static ref ridge: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-7, &[1f64, 2f64, 1f64, 0f64, 0f64, 0f64],);
        pub static ref offset: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-3, &[1f64; 4],);
        pub static ref aquifer_barrier: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-3, &[1f64,],);
        pub static ref aquifer_barrier_floodedness: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-7, &[1f64,],);
        pub static ref aquifer_lava: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-1, &[1f64,],);
        pub static ref aquifer_fluid_level_spread: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-5, &[1f64,],);
        pub static ref pillar: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-7, &[1f64; 2],);
        pub static ref pillar_rareness: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-8, &[1f64,],);
        pub static ref pillar_thickness: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-8, &[1f64,],);
        pub static ref spaghetti_2d: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-7, &[1f64,],);
        pub static ref spaghetti_2d_elevation: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-8, &[1f64,],);
        pub static ref spaghetti_2d_modulator: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-11, &[1f64,],);
        pub static ref spaghetti_2d_thickness: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-11, &[1f64,],);
        pub static ref spaghetti_3d_1: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-7, &[1f64,],);
        pub static ref spaghetti_3d_2: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-7, &[1f64,],);
        pub static ref spaghetti_3d_rarity: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-11, &[1f64,],);
        pub static ref spaghetti_3d_thickness: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-8, &[1f64,],);
        pub static ref spaghetti_roughness: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-5, &[1f64,],);
        pub static ref spaghetti_roughness_modulator: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-8, &[1f64,],);
        pub static ref cave_entrance: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-7, &[0.4f64, 0.5f64, 1f64],);
        pub static ref cave_layer: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-8, &[1f64,],);
        pub static ref cave_cheese: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(
                -8,
                &[0.5f64, 1f64, 2f64, 1f64, 2f64, 1f64, 0f64, 2f64, 0f64,],
            );
        pub static ref ore_veininess: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-8, &[1f64,],);
        pub static ref ore_vein_a: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-7, &[1f64,],);
        pub static ref ore_vein_b: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-7, &[1f64,],);
        pub static ref ore_gap: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-5, &[1f64,],);
        pub static ref noodle: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-8, &[1f64,],);
        pub static ref noodle_thickness: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-8, &[1f64,],);
        pub static ref noodle_ridge_a: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-7, &[1f64,],);
        pub static ref noodle_ridge_b: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-7, &[1f64,],);
        pub static ref jagged: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-16, &[1f64; 16],);
        pub static ref surface: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-6, &[1f64; 3],);
        pub static ref surface_secondary: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-6, &[1f64, 1f64, 0f64, 1f64,],);
        pub static ref clay_bands_offset: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-8, &[1f64,],);
        pub static ref badlands_pillar: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-2, &[1f64; 4],);
        pub static ref badlands_pillar_roof: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-8, &[1f64,],);
        pub static ref badlands_surface: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-6, &[1f64; 3],);
        pub static ref iceberg_pillar: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-6, &[1f64; 4],);
        pub static ref iceberg_pillar_roof: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-3, &[1f64,],);
        pub static ref iceberg_surface: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-6, &[1f64; 3],);
        pub static ref surface_swamp: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-2, &[1f64,],);
        pub static ref calcite: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-9, &[1f64; 4],);
        pub static ref gravel: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-8, &[1f64; 4],);
        pub static ref powder_snow: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-6, &[1f64; 4],);
        pub static ref packed_ice: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-7, &[1f64; 4],);
        pub static ref ice: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-4, &[1f64; 4],);
        pub static ref soul_sand_layer: DoublePerlinNoiseParameters<'static> =
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
            );
        pub static ref gravel_layer: DoublePerlinNoiseParameters<'static> =
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
            );
        pub static ref patch: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(
                -5,
                &[1f64, 0f64, 0f64, 0f64, 0f64, 0.013333333333333334f64,],
            );
        pub static ref netherrack: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-3, &[1f64, 0f64, 0f64, 0.35f64,],);
        pub static ref nether_wart: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-3, &[1f64, 0f64, 0f64, 0.9f64,],);
        pub static ref nether_state_selector: DoublePerlinNoiseParameters<'static> =
            DoublePerlinNoiseParameters::new(-4, &[1f64,],);
    }
}

pub fn lerp(delta: f64, start: f64, end: f64) -> f64 {
    start + delta * (end - start)
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
