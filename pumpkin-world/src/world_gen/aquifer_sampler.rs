use enum_dispatch::enum_dispatch;
use num_traits::PrimInt;
use pumpkin_core::{
    math::{floor_div, vector2::Vector2, vector3::Vector3},
    random::RandomDeriver,
};

use crate::block::BlockState;

use super::{
    chunk_noise::{ChunkNoiseDensityFunctions, ChunkNoiseState, LAVA_BLOCK, WATER_BLOCK},
    noise::{
        clamped_map,
        density::{
            component_functions::ComponentReference, NoisePos, NoisePosImpl, UnblendedNoisePos,
        },
        map,
    },
    positions::{block_pos, chunk_pos, MIN_HEIGHT_CELL},
    proto_chunk::StandardChunkFluidLevelSampler,
    section_coords,
};

#[derive(Clone)]
pub struct FluidLevel {
    max_y: i32,
    state: BlockState,
}

impl FluidLevel {
    pub fn new(max_y: i32, state: BlockState) -> Self {
        Self { max_y, state }
    }

    pub fn max_y_exclusive(&self) -> i32 {
        self.max_y
    }

    fn get_block_state(&self, y: i32) -> BlockState {
        if y < self.max_y {
            self.state
        } else {
            BlockState::AIR
        }
    }
}

#[enum_dispatch(FluidLevelSamplerImpl)]
pub enum FluidLevelSampler {
    Static(StaticFluidLevelSampler),
    Chunk(StandardChunkFluidLevelSampler),
}

pub struct StaticFluidLevelSampler {
    y: i32,
    state: BlockState,
}

impl StaticFluidLevelSampler {
    pub fn new(y: i32, state: BlockState) -> Self {
        Self { y, state }
    }
}

impl FluidLevelSamplerImpl for StaticFluidLevelSampler {
    fn get_fluid_level(&self, _x: i32, _y: i32, _z: i32) -> FluidLevel {
        FluidLevel::new(self.y, self.state)
    }
}

#[enum_dispatch]
pub trait FluidLevelSamplerImpl {
    fn get_fluid_level(&self, x: i32, y: i32, z: i32) -> FluidLevel;
}

#[enum_dispatch(AquiferSamplerImpl)]
pub enum AquiferSampler {
    SeaLevel(SeaLevelAquiferSampler),
    Aquifier(WorldAquiferSampler),
}

pub struct WorldAquiferSampler {
    barrier_noise: Box<dyn ComponentReference<ChunkNoiseState>>,
    fluid_level_floodedness: Box<dyn ComponentReference<ChunkNoiseState>>,
    fluid_level_spread: Box<dyn ComponentReference<ChunkNoiseState>>,
    fluid_type: Box<dyn ComponentReference<ChunkNoiseState>>,
    erosion: Box<dyn ComponentReference<ChunkNoiseState>>,
    depth: Box<dyn ComponentReference<ChunkNoiseState>>,
    function: Box<dyn ComponentReference<ChunkNoiseState>>,
    random_deriver: RandomDeriver,
    fluid_level: FluidLevelSampler,
    start_x: i32,
    size_x: usize,
    start_y: i8,
    start_z: i32,
    size_z: usize,
    levels: Box<[Option<FluidLevel>]>,
    packed_positions: Box<[i64]>,
}

impl WorldAquiferSampler {
    const CHUNK_POS_OFFSETS: [Vector2<i8>; 13] = [
        Vector2::new(0, 0),
        Vector2::new(-2, -1),
        Vector2::new(-1, -1),
        Vector2::new(0, -1),
        Vector2::new(1, -1),
        Vector2::new(-3, 0),
        Vector2::new(-2, 0),
        Vector2::new(-1, 0),
        Vector2::new(1, 0),
        Vector2::new(-2, 1),
        Vector2::new(-1, 1),
        Vector2::new(0, 1),
        Vector2::new(1, 1),
    ];

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chunk_pos: Vector2<i32>,
        barrier_noise: Box<dyn ComponentReference<ChunkNoiseState>>,
        fluid_level_floodedness: Box<dyn ComponentReference<ChunkNoiseState>>,
        fluid_level_spread: Box<dyn ComponentReference<ChunkNoiseState>>,
        fluid_type: Box<dyn ComponentReference<ChunkNoiseState>>,
        erosion: Box<dyn ComponentReference<ChunkNoiseState>>,
        depth: Box<dyn ComponentReference<ChunkNoiseState>>,
        function: Box<dyn ComponentReference<ChunkNoiseState>>,
        random_deriver: RandomDeriver,
        minimum_y: i8,
        height: u16,
        fluid_level: FluidLevelSampler,
    ) -> Self {
        let start_x = Self::get_local_x(chunk_pos::start_block_x(&chunk_pos)) - 1;
        let end_x = Self::get_local_x(chunk_pos::end_block_x(&chunk_pos)) + 1;
        let size_x = (end_x - start_x) as usize + 1;

        let start_y = Self::get_local_y(minimum_y) - 1;
        let end_y = Self::get_local_y(minimum_y as i32 + height as i32) + 1;
        let size_y = (end_y - start_y as i32) as usize + 1;

        let start_z = Self::get_local_z(chunk_pos::start_block_z(&chunk_pos)) - 1;
        let end_z = Self::get_local_z(chunk_pos::end_block_z(&chunk_pos)) + 1;
        let size_z = (end_z - start_z) as usize + 1;

        let cache_size = size_x * size_y * size_z;

        let mut packed_positions = vec![0; cache_size];

        for offset_x in 0..size_x {
            for offset_y in 0..size_y {
                for offset_z in 0..size_z {
                    let x = start_x + offset_x as i32;
                    let y = start_y as i32 + offset_y as i32;
                    let z = start_z + offset_z as i32;

                    let mut random = random_deriver.split_pos(x, y, z);
                    let rand_x = x * 16 + random.next_bounded_i32(10);
                    let rand_y = y * 12 + random.next_bounded_i32(9);
                    let rand_z = z * 16 + random.next_bounded_i32(10);

                    let index = (offset_y * size_z + offset_z) * size_x + offset_x;
                    packed_positions[index] =
                        block_pos::packed(&Vector3::new(rand_x, rand_y, rand_z));
                }
            }
        }

        Self {
            barrier_noise,
            fluid_level_floodedness,
            fluid_level_spread,
            fluid_type,
            erosion,
            depth,
            function,
            random_deriver,
            fluid_level,
            start_x,
            size_x,
            start_y,
            start_z,
            size_z,
            levels: vec![None; cache_size].into(),
            packed_positions: packed_positions.into(),
        }
    }

    #[inline]
    fn index(&self, x: i32, y: i32, z: i32) -> usize {
        let i = (x - self.start_x) as usize;
        let j = (y - self.start_y as i32) as usize;
        let k = (z - self.start_z) as usize;

        (j * self.size_z + k) * self.size_x + i
    }

    #[inline]
    fn max_distance(i: i32, a: i32) -> f64 {
        1f64 - ((a - i).abs() as f64) / 25f64
    }

    fn calculate_density(
        &mut self,
        pos: &NoisePos,
        barrier_sample: f64,
        level_1: FluidLevel,
        level_2: FluidLevel,
    ) -> f64 {
        let y = pos.y();
        let block_state1 = level_1.get_block_state(y);
        let block_state2 = level_2.get_block_state(y);

        if (!block_state1.of_block(LAVA_BLOCK.block_id)
            || !block_state2.of_block(WATER_BLOCK.block_id))
            && (!block_state1.of_block(WATER_BLOCK.block_id)
                || !block_state2.of_block(LAVA_BLOCK.block_id))
        {
            let level_diff = (level_1.max_y - level_2.max_y).abs();
            if level_diff == 0 {
                0f64
            } else {
                let avg_level = 0.5f64 * (level_1.max_y + level_2.max_y) as f64;
                let scaled_level = y as f64 + 0.5f64 - avg_level;
                let halved_diff = level_diff as f64 / 2f64;

                let o = halved_diff - scaled_level.abs();
                let q = if scaled_level > 0f64 {
                    if o > 0f64 {
                        o / 1.5f64
                    } else {
                        o / 2.5f64
                    }
                } else {
                    let p = 3f64 + o;
                    if p > 0f64 {
                        p / 3f64
                    } else {
                        p / 10f64
                    }
                };

                let r = if (-2f64..=2f64).contains(&q) {
                    barrier_sample
                } else {
                    0f64
                };

                2f64 * (r + q)
            }
        } else {
            2f64
        }
    }

    fn get_water_level(
        &mut self,
        packed: i64,
        height_estimator: &mut ChunkNoiseDensityFunctions,
        env: &ChunkNoiseState,
    ) -> FluidLevel {
        let x = block_pos::unpack_x(packed);
        let y = block_pos::unpack_y(packed);
        let z = block_pos::unpack_z(packed);

        let local_x = Self::get_local_x(x);
        let local_y = Self::get_local_y(y);
        let local_z = Self::get_local_z(z);

        let index = self.index(local_x, local_y, local_z);
        if let Some(level) = &self.levels[index] {
            level.clone()
        } else {
            let fluid_level = self.get_fluid_level(x, y, z, height_estimator, env);
            self.levels[index] = Some(fluid_level.clone());
            fluid_level
        }
    }

    fn get_fluid_level(
        &mut self,
        block_x: i32,
        block_y: i32,
        block_z: i32,
        height_estimator: &mut ChunkNoiseDensityFunctions,
        env: &ChunkNoiseState,
    ) -> FluidLevel {
        let fluid_level = self.fluid_level.get_fluid_level(block_x, block_y, block_z);
        let j = block_y + 12;
        let k = block_y - 12;
        let mut bl = false;
        let mut min_surface_estimate = i32::MAX;

        for offset in Self::CHUNK_POS_OFFSETS {
            let x = block_x + section_coords::section_to_block(offset.x as i32);
            let z = block_z + section_coords::section_to_block(offset.z as i32);

            let n = height_estimator.estimate_surface_height(env, x, z);
            let o = n + 8;
            let bl2 = offset.x == 0 && offset.z == 0;

            if bl2 && k > o {
                return fluid_level;
            }

            let bl3 = j > o;
            if bl3 || bl2 {
                let fluid_level = self.fluid_level.get_fluid_level(x, o, z);
                if !fluid_level.get_block_state(o).is_air() {
                    if bl2 {
                        bl = true;
                    }

                    if bl3 {
                        return fluid_level;
                    }
                }
            }

            min_surface_estimate = min_surface_estimate.min(n);
        }

        let p = self.get_fluid_block_y(
            block_x,
            block_y,
            block_z,
            fluid_level.clone(),
            min_surface_estimate,
            bl,
            env,
        );
        FluidLevel::new(
            p,
            self.get_fluid_block_state(block_x, block_y, block_z, fluid_level, p, env),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn get_fluid_block_y(
        &mut self,
        block_x: i32,
        block_y: i32,
        block_z: i32,
        default_level: FluidLevel,
        surface_height_estimate: i32,
        map_y: bool,
        env: &ChunkNoiseState,
    ) -> i32 {
        let pos = NoisePos::Unblended(UnblendedNoisePos::new(block_x, block_y, block_z));

        let is_deep_dark = self.erosion.sample_mut(&pos, env) < -0.225f32 as f64
            && self.depth.sample_mut(&pos, env) > 0.9f32 as f64;

        let (d, e) = if is_deep_dark {
            (-1f64, -1f64)
        } else {
            let top_y = surface_height_estimate + 8 - block_y;
            let f = if map_y {
                clamped_map(top_y as f64, 0f64, 64f64, 1f64, 0f64)
            } else {
                0f64
            };

            let g = self
                .fluid_level_floodedness
                .sample_mut(&pos, env)
                .clamp(-1f64, 1f64);
            let h = map(f, 1f64, 0f64, -0.3f64, 0.8f64);
            let k = map(f, 1f64, 0f64, -0.8f64, 0.4f64);

            (g - k, g - h)
        };

        if e > 0f64 {
            default_level.max_y
        } else if d > 0f64 {
            self.get_noise_based_fluid_level(
                block_x,
                block_y,
                block_z,
                surface_height_estimate,
                env,
            )
        } else {
            MIN_HEIGHT_CELL
        }
    }

    fn get_noise_based_fluid_level(
        &mut self,
        block_x: i32,
        block_y: i32,
        block_z: i32,
        surface_height_estimate: i32,
        env: &ChunkNoiseState,
    ) -> i32 {
        let x = floor_div(block_x, 16);
        let y = floor_div(block_y, 40);
        let z = floor_div(block_z, 16);

        let local_y = y * 40 + 20;
        let sample = self
            .fluid_level_spread
            .sample_mut(&NoisePos::Unblended(UnblendedNoisePos::new(x, y, z)), env)
            * 10f64;
        let to_nearest_multiple_of_three = (sample / 3f64).floor() as i32 * 3;
        let local_height = to_nearest_multiple_of_three + local_y;

        surface_height_estimate.min(local_height)
    }

    fn get_fluid_block_state(
        &mut self,
        block_x: i32,
        block_y: i32,
        block_z: i32,
        default_level: FluidLevel,
        level: i32,
        env: &ChunkNoiseState,
    ) -> BlockState {
        if level <= -10
            && level != MIN_HEIGHT_CELL
            && !default_level.state.of_block(LAVA_BLOCK.block_id)
        {
            let x = floor_div(block_x, 64);
            let y = floor_div(block_y, 40);
            let z = floor_div(block_z, 64);

            let sample = self
                .fluid_type
                .sample_mut(&NoisePos::Unblended(UnblendedNoisePos::new(x, y, z)), env);

            if sample.abs() > 0.3f64 {
                return *LAVA_BLOCK;
            }
        }

        default_level.state
    }

    #[inline]
    fn get_local_x<T>(x: T) -> T
    where
        T: PrimInt + From<i8>,
    {
        floor_div(x, 16.into())
    }

    #[inline]
    fn get_local_y<T>(y: T) -> T
    where
        T: PrimInt + From<i8>,
    {
        floor_div(y, 12.into())
    }

    #[inline]
    fn get_local_z<T>(z: T) -> T
    where
        T: PrimInt + From<i8>,
    {
        floor_div(z, 16.into())
    }

    fn apply_internal(
        &mut self,
        pos: &NoisePos,
        state: &ChunkNoiseState,
        height_estimator: &mut ChunkNoiseDensityFunctions,
        density: f64,
    ) -> Option<BlockState> {
        if density > 0f64 {
            None
        } else {
            let i = pos.x();
            let j = pos.y();
            let k = pos.z();

            let fluid_level = self.fluid_level.get_fluid_level(i, j, k);
            if fluid_level.get_block_state(j).of_block(LAVA_BLOCK.block_id) {
                Some(*LAVA_BLOCK)
            } else {
                let scaled_x = floor_div(i - 5, 16);
                let scaled_y = floor_div(j + 1, 12);
                let scaled_z = floor_div(k - 5, 16);

                // The 3 closest positions, closest to furthest
                let mut hypot_packed_block = [(0, i32::MAX); 3];
                for offset_y in -1..=1 {
                    for offset_x in 0..=1 {
                        for offset_z in 0..=1 {
                            let x_pos = scaled_x + offset_x;
                            let y_pos = scaled_y + offset_y;
                            let z_pos = scaled_z + offset_z;
                            let index = self.index(x_pos, y_pos, z_pos);

                            let packed_random = self.packed_positions[index];

                            let local_x = block_pos::unpack_x(packed_random) - i;
                            let local_y = block_pos::unpack_y(packed_random) - j;
                            let local_z = block_pos::unpack_z(packed_random) - k;

                            let hypot_squared =
                                local_x * local_x + local_y * local_y + local_z * local_z;

                            if hypot_packed_block[2].1 >= hypot_squared {
                                hypot_packed_block[2] = (packed_random, hypot_squared);
                            }

                            if hypot_packed_block[1].1 >= hypot_squared {
                                hypot_packed_block[2] = hypot_packed_block[1];
                                hypot_packed_block[1] = (packed_random, hypot_squared);
                            }

                            if hypot_packed_block[0].1 >= hypot_squared {
                                hypot_packed_block[1] = hypot_packed_block[0];
                                hypot_packed_block[0] = (packed_random, hypot_squared);
                            }
                        }
                    }
                }

                let fluid_level2 =
                    self.get_water_level(hypot_packed_block[0].0, height_estimator, state);
                let d = Self::max_distance(hypot_packed_block[0].1, hypot_packed_block[1].1);
                let block_state = fluid_level2.get_block_state(j);

                if d <= 0f64 {
                    // TODO: Handle fluid tick

                    Some(block_state)
                } else if block_state.of_block(WATER_BLOCK.block_id)
                    && self
                        .fluid_level
                        .get_fluid_level(i, j - 1, k)
                        .get_block_state(j - 1)
                        .of_block(LAVA_BLOCK.block_id)
                {
                    Some(block_state)
                } else {
                    let barrier_sample = self.barrier_noise.sample_mut(pos, state);
                    let fluid_level3 =
                        self.get_water_level(hypot_packed_block[1].0, height_estimator, state);
                    let e = d * self.calculate_density(
                        pos,
                        barrier_sample,
                        fluid_level2.clone(),
                        fluid_level3.clone(),
                    );

                    if density + e > 0f64 {
                        None
                    } else {
                        let fluid_level4 =
                            self.get_water_level(hypot_packed_block[2].0, height_estimator, state);
                        let f =
                            Self::max_distance(hypot_packed_block[0].1, hypot_packed_block[2].1);
                        if f > 0f64 {
                            let g = d
                                * f
                                * self.calculate_density(
                                    pos,
                                    barrier_sample,
                                    fluid_level2,
                                    fluid_level4.clone(),
                                );
                            if density + g > 0f64 {
                                return None;
                            }
                        }

                        let g =
                            Self::max_distance(hypot_packed_block[1].1, hypot_packed_block[2].1);
                        if g > 0f64 {
                            let h = d
                                * g
                                * self.calculate_density(
                                    pos,
                                    barrier_sample,
                                    fluid_level3,
                                    fluid_level4,
                                );
                            if density + h > 0f64 {
                                return None;
                            }
                        }

                        //TODO Handle fluid tick

                        Some(block_state)
                    }
                }
            }
        }
    }
}

impl AquiferSamplerImpl for WorldAquiferSampler {
    #[inline]
    fn apply(
        &mut self,
        pos: &NoisePos,
        state: &ChunkNoiseState,
        height_estimator: &mut ChunkNoiseDensityFunctions,
    ) -> Option<BlockState> {
        let density = self.function.sample_mut(pos, state);
        self.apply_internal(pos, state, height_estimator, density)
    }
}

pub struct SeaLevelAquiferSampler {
    level_sampler: FluidLevelSampler,
    function: Box<dyn ComponentReference<ChunkNoiseState>>,
}

impl SeaLevelAquiferSampler {
    pub fn new(
        level_sampler: FluidLevelSampler,
        function: Box<dyn ComponentReference<ChunkNoiseState>>,
    ) -> Self {
        Self {
            level_sampler,
            function,
        }
    }
}

impl AquiferSamplerImpl for SeaLevelAquiferSampler {
    fn apply(
        &mut self,
        pos: &NoisePos,
        state: &ChunkNoiseState,
        _height_estimator: &mut ChunkNoiseDensityFunctions,
    ) -> Option<BlockState> {
        let sample = self.function.sample_mut(pos, state);
        //log::debug!("Aquifer sample {:?}: {}", &pos, sample);
        if sample > 0f64 {
            None
        } else {
            Some(
                self.level_sampler
                    .get_fluid_level(pos.x(), pos.y(), pos.z())
                    .get_block_state(pos.y()),
            )
        }
    }
}

#[enum_dispatch]
pub trait AquiferSamplerImpl {
    fn apply(
        &mut self,
        pos: &NoisePos,
        state: &ChunkNoiseState,
        height_estimator: &mut ChunkNoiseDensityFunctions,
    ) -> Option<BlockState>;
}

#[cfg(test)]
mod test {
    use pumpkin_core::math::vector2::Vector2;

    use crate::{
        block::BlockState,
        world_gen::{
            chunk_noise::{
                BlockStateSampler, ChunkNoiseDensityFunctions, ChunkNoiseGenerator,
                ChunkNoiseState, LAVA_BLOCK, WATER_BLOCK,
            },
            generation_shapes::GenerationShape,
            noise::{
                config::NoiseConfig,
                density::{NoisePos, UnblendedNoisePos},
                router::OVERWORLD_NOISE_ROUTER,
            },
            positions::chunk_pos,
            proto_chunk::StandardChunkFluidLevelSampler,
        },
    };

    use super::{AquiferSampler, FluidLevel, FluidLevelSampler, WorldAquiferSampler};

    fn create_aquifer() -> (
        WorldAquiferSampler,
        ChunkNoiseDensityFunctions,
        ChunkNoiseState,
    ) {
        let shape = GenerationShape::SURFACE;
        let chunk_pos = Vector2::new(7, 4);
        let config = NoiseConfig::new(0, &OVERWORLD_NOISE_ROUTER);
        let sampler = FluidLevelSampler::Chunk(StandardChunkFluidLevelSampler::new(
            FluidLevel::new(63, *WATER_BLOCK),
            FluidLevel::new(-54, *LAVA_BLOCK),
        ));
        let noise = ChunkNoiseGenerator::new(
            16 / shape.horizontal_cell_block_count(),
            chunk_pos::start_block_x(&chunk_pos),
            chunk_pos::start_block_z(&chunk_pos),
            shape,
            &config,
            sampler,
            true,
            true,
        );
        let sampler = match noise.state_sampler {
            BlockStateSampler::Chained(chained) => chained,
            _ => unreachable!(),
        };
        let mut samplers = sampler.samplers;
        samplers.truncate(1);
        let sampler = match samplers.pop().unwrap() {
            BlockStateSampler::Aquifer(aquifer) => aquifer,
            _ => unreachable!(),
        };
        let aquifer = match sampler {
            AquiferSampler::Aquifier(aquifer) => aquifer,
            _ => unreachable!(),
        };
        (aquifer, noise.density_functions, noise.shared)
    }

    #[test]
    fn test_get_fluid_block_state() {
        let (mut aquifer, _, env) = create_aquifer();
        let level = FluidLevel::new(0, *WATER_BLOCK);

        let values = [
            ((-100, -100, -100), *WATER_BLOCK),
            ((-100, -100, -50), *LAVA_BLOCK),
            ((-100, -100, 0), *WATER_BLOCK),
            ((-100, -100, 50), *WATER_BLOCK),
            ((-100, -100, 100), *WATER_BLOCK),
            ((-100, -50, -100), *WATER_BLOCK),
            ((-100, -50, -50), *LAVA_BLOCK),
            ((-100, -50, 0), *LAVA_BLOCK),
            ((-100, -50, 50), *LAVA_BLOCK),
            ((-100, -50, 100), *WATER_BLOCK),
            ((-100, 0, -100), *WATER_BLOCK),
            ((-100, 0, -50), *WATER_BLOCK),
            ((-100, 0, 0), *WATER_BLOCK),
            ((-100, 0, 50), *WATER_BLOCK),
            ((-100, 0, 100), *WATER_BLOCK),
            ((-100, 50, -100), *WATER_BLOCK),
            ((-100, 50, -50), *WATER_BLOCK),
            ((-100, 50, 0), *WATER_BLOCK),
            ((-100, 50, 50), *WATER_BLOCK),
            ((-100, 50, 100), *WATER_BLOCK),
            ((-100, 100, -100), *WATER_BLOCK),
            ((-100, 100, -50), *WATER_BLOCK),
            ((-100, 100, 0), *WATER_BLOCK),
            ((-100, 100, 50), *WATER_BLOCK),
            ((-100, 100, 100), *WATER_BLOCK),
            ((-50, -100, -100), *WATER_BLOCK),
            ((-50, -100, -50), *WATER_BLOCK),
            ((-50, -100, 0), *LAVA_BLOCK),
            ((-50, -100, 50), *LAVA_BLOCK),
            ((-50, -100, 100), *WATER_BLOCK),
            ((-50, -50, -100), *WATER_BLOCK),
            ((-50, -50, -50), *WATER_BLOCK),
            ((-50, -50, 0), *WATER_BLOCK),
            ((-50, -50, 50), *WATER_BLOCK),
            ((-50, -50, 100), *WATER_BLOCK),
            ((-50, 0, -100), *LAVA_BLOCK),
            ((-50, 0, -50), *WATER_BLOCK),
            ((-50, 0, 0), *WATER_BLOCK),
            ((-50, 0, 50), *WATER_BLOCK),
            ((-50, 0, 100), *WATER_BLOCK),
            ((-50, 50, -100), *WATER_BLOCK),
            ((-50, 50, -50), *WATER_BLOCK),
            ((-50, 50, 0), *LAVA_BLOCK),
            ((-50, 50, 50), *LAVA_BLOCK),
            ((-50, 50, 100), *WATER_BLOCK),
            ((-50, 100, -100), *WATER_BLOCK),
            ((-50, 100, -50), *WATER_BLOCK),
            ((-50, 100, 0), *LAVA_BLOCK),
            ((-50, 100, 50), *LAVA_BLOCK),
            ((-50, 100, 100), *LAVA_BLOCK),
            ((0, -100, -100), *WATER_BLOCK),
            ((0, -100, -50), *LAVA_BLOCK),
            ((0, -100, 0), *LAVA_BLOCK),
            ((0, -100, 50), *LAVA_BLOCK),
            ((0, -100, 100), *WATER_BLOCK),
            ((0, -50, -100), *WATER_BLOCK),
            ((0, -50, -50), *WATER_BLOCK),
            ((0, -50, 0), *WATER_BLOCK),
            ((0, -50, 50), *WATER_BLOCK),
            ((0, -50, 100), *WATER_BLOCK),
            ((0, 0, -100), *LAVA_BLOCK),
            ((0, 0, -50), *LAVA_BLOCK),
            ((0, 0, 0), *WATER_BLOCK),
            ((0, 0, 50), *WATER_BLOCK),
            ((0, 0, 100), *WATER_BLOCK),
            ((0, 50, -100), *WATER_BLOCK),
            ((0, 50, -50), *WATER_BLOCK),
            ((0, 50, 0), *WATER_BLOCK),
            ((0, 50, 50), *WATER_BLOCK),
            ((0, 50, 100), *WATER_BLOCK),
            ((0, 100, -100), *WATER_BLOCK),
            ((0, 100, -50), *LAVA_BLOCK),
            ((0, 100, 0), *WATER_BLOCK),
            ((0, 100, 50), *WATER_BLOCK),
            ((0, 100, 100), *WATER_BLOCK),
            ((50, -100, -100), *WATER_BLOCK),
            ((50, -100, -50), *LAVA_BLOCK),
            ((50, -100, 0), *LAVA_BLOCK),
            ((50, -100, 50), *LAVA_BLOCK),
            ((50, -100, 100), *WATER_BLOCK),
            ((50, -50, -100), *WATER_BLOCK),
            ((50, -50, -50), *WATER_BLOCK),
            ((50, -50, 0), *WATER_BLOCK),
            ((50, -50, 50), *WATER_BLOCK),
            ((50, -50, 100), *WATER_BLOCK),
            ((50, 0, -100), *LAVA_BLOCK),
            ((50, 0, -50), *LAVA_BLOCK),
            ((50, 0, 0), *WATER_BLOCK),
            ((50, 0, 50), *WATER_BLOCK),
            ((50, 0, 100), *WATER_BLOCK),
            ((50, 50, -100), *WATER_BLOCK),
            ((50, 50, -50), *WATER_BLOCK),
            ((50, 50, 0), *WATER_BLOCK),
            ((50, 50, 50), *WATER_BLOCK),
            ((50, 50, 100), *WATER_BLOCK),
            ((50, 100, -100), *WATER_BLOCK),
            ((50, 100, -50), *LAVA_BLOCK),
            ((50, 100, 0), *WATER_BLOCK),
            ((50, 100, 50), *WATER_BLOCK),
            ((50, 100, 100), *WATER_BLOCK),
            ((100, -100, -100), *WATER_BLOCK),
            ((100, -100, -50), *LAVA_BLOCK),
            ((100, -100, 0), *WATER_BLOCK),
            ((100, -100, 50), *WATER_BLOCK),
            ((100, -100, 100), *WATER_BLOCK),
            ((100, -50, -100), *LAVA_BLOCK),
            ((100, -50, -50), *LAVA_BLOCK),
            ((100, -50, 0), *LAVA_BLOCK),
            ((100, -50, 50), *LAVA_BLOCK),
            ((100, -50, 100), *LAVA_BLOCK),
            ((100, 0, -100), *WATER_BLOCK),
            ((100, 0, -50), *LAVA_BLOCK),
            ((100, 0, 0), *WATER_BLOCK),
            ((100, 0, 50), *WATER_BLOCK),
            ((100, 0, 100), *LAVA_BLOCK),
            ((100, 50, -100), *WATER_BLOCK),
            ((100, 50, -50), *WATER_BLOCK),
            ((100, 50, 0), *WATER_BLOCK),
            ((100, 50, 50), *WATER_BLOCK),
            ((100, 50, 100), *WATER_BLOCK),
            ((100, 100, -100), *LAVA_BLOCK),
            ((100, 100, -50), *LAVA_BLOCK),
            ((100, 100, 0), *WATER_BLOCK),
            ((100, 100, 50), *WATER_BLOCK),
            ((100, 100, 100), *WATER_BLOCK),
        ];

        for ((x, y, z), result) in values {
            assert_eq!(
                aquifer.get_fluid_block_state(x, y, z, level.clone(), -10, &env),
                result
            );
        }
    }

    #[test]
    fn test_get_noise_based_fluid_level() {
        let (mut aquifer, _, env) = create_aquifer();
        let values = [
            ((-100, -100, -100), -103),
            ((-100, -100, -50), -103),
            ((-100, -100, 0), -103),
            ((-100, -100, 50), -103),
            ((-100, -100, 100), -103),
            ((-100, -50, -100), -63),
            ((-100, -50, -50), -63),
            ((-100, -50, 0), -63),
            ((-100, -50, 50), -63),
            ((-100, -50, 100), -63),
            ((-100, 0, -100), 17),
            ((-100, 0, -50), 17),
            ((-100, 0, 0), 17),
            ((-100, 0, 50), 17),
            ((-100, 0, 100), 17),
            ((-100, 50, -100), 57),
            ((-100, 50, -50), 57),
            ((-100, 50, 0), 57),
            ((-100, 50, 50), 57),
            ((-100, 50, 100), 57),
            ((-100, 100, -100), 97),
            ((-100, 100, -50), 97),
            ((-100, 100, 0), 97),
            ((-100, 100, 50), 97),
            ((-100, 100, 100), 97),
            ((-50, -100, -100), -103),
            ((-50, -100, -50), -103),
            ((-50, -100, 0), -103),
            ((-50, -100, 50), -103),
            ((-50, -100, 100), -100),
            ((-50, -50, -100), -63),
            ((-50, -50, -50), -63),
            ((-50, -50, 0), -63),
            ((-50, -50, 50), -63),
            ((-50, -50, 100), -60),
            ((-50, 0, -100), 17),
            ((-50, 0, -50), 17),
            ((-50, 0, 0), 17),
            ((-50, 0, 50), 17),
            ((-50, 0, 100), 20),
            ((-50, 50, -100), 57),
            ((-50, 50, -50), 57),
            ((-50, 50, 0), 57),
            ((-50, 50, 50), 57),
            ((-50, 50, 100), 60),
            ((-50, 100, -100), 97),
            ((-50, 100, -50), 97),
            ((-50, 100, 0), 97),
            ((-50, 100, 50), 97),
            ((-50, 100, 100), 100),
            ((0, -100, -100), -103),
            ((0, -100, -50), -103),
            ((0, -100, 0), -103),
            ((0, -100, 50), -100),
            ((0, -100, 100), -100),
            ((0, -50, -100), -63),
            ((0, -50, -50), -63),
            ((0, -50, 0), -63),
            ((0, -50, 50), -60),
            ((0, -50, 100), -60),
            ((0, 0, -100), 17),
            ((0, 0, -50), 17),
            ((0, 0, 0), 17),
            ((0, 0, 50), 20),
            ((0, 0, 100), 20),
            ((0, 50, -100), 57),
            ((0, 50, -50), 57),
            ((0, 50, 0), 57),
            ((0, 50, 50), 60),
            ((0, 50, 100), 60),
            ((0, 100, -100), 97),
            ((0, 100, -50), 97),
            ((0, 100, 0), 97),
            ((0, 100, 50), 100),
            ((0, 100, 100), 100),
            ((50, -100, -100), -103),
            ((50, -100, -50), -103),
            ((50, -100, 0), -103),
            ((50, -100, 50), -100),
            ((50, -100, 100), -100),
            ((50, -50, -100), -63),
            ((50, -50, -50), -63),
            ((50, -50, 0), -63),
            ((50, -50, 50), -60),
            ((50, -50, 100), -60),
            ((50, 0, -100), 17),
            ((50, 0, -50), 17),
            ((50, 0, 0), 17),
            ((50, 0, 50), 20),
            ((50, 0, 100), 20),
            ((50, 50, -100), 57),
            ((50, 50, -50), 57),
            ((50, 50, 0), 57),
            ((50, 50, 50), 60),
            ((50, 50, 100), 60),
            ((50, 100, -100), 97),
            ((50, 100, -50), 97),
            ((50, 100, 0), 97),
            ((50, 100, 50), 100),
            ((50, 100, 100), 100),
            ((100, -100, -100), -103),
            ((100, -100, -50), -103),
            ((100, -100, 0), -103),
            ((100, -100, 50), -103),
            ((100, -100, 100), -100),
            ((100, -50, -100), -63),
            ((100, -50, -50), -63),
            ((100, -50, 0), -63),
            ((100, -50, 50), -63),
            ((100, -50, 100), -60),
            ((100, 0, -100), 17),
            ((100, 0, -50), 17),
            ((100, 0, 0), 17),
            ((100, 0, 50), 17),
            ((100, 0, 100), 20),
            ((100, 50, -100), 57),
            ((100, 50, -50), 57),
            ((100, 50, 0), 57),
            ((100, 50, 50), 57),
            ((100, 50, 100), 60),
            ((100, 100, -100), 97),
            ((100, 100, -50), 97),
            ((100, 100, 0), 97),
            ((100, 100, 50), 97),
            ((100, 100, 100), 100),
        ];

        for ((x, y, z), result) in values {
            assert_eq!(
                aquifer.get_noise_based_fluid_level(x, y, z, 200, &env),
                result
            );
        }
    }

    #[test]
    fn test_get_fluid_block_y() {
        let (mut aquifer, _, env) = create_aquifer();
        let level = FluidLevel::new(0, *WATER_BLOCK);
        let values = [
            ((-100, -100, -100), -32512),
            ((-100, -100, -50), -32512),
            ((-100, -100, 0), -32512),
            ((-100, -100, 50), -32512),
            ((-100, -100, 100), -32512),
            ((-100, -50, -100), -32512),
            ((-100, -50, -50), -63),
            ((-100, -50, 0), -63),
            ((-100, -50, 50), -63),
            ((-100, -50, 100), -32512),
            ((-100, 0, -100), -32512),
            ((-100, 0, -50), -32512),
            ((-100, 0, 0), -32512),
            ((-100, 0, 50), -32512),
            ((-100, 0, 100), -32512),
            ((-100, 50, -100), 57),
            ((-100, 50, -50), 57),
            ((-100, 50, 0), 57),
            ((-100, 50, 50), 57),
            ((-100, 50, 100), 57),
            ((-100, 100, -100), 80),
            ((-100, 100, -50), 0),
            ((-100, 100, 0), 0),
            ((-100, 100, 50), 0),
            ((-100, 100, 100), 0),
            ((-50, -100, -100), -32512),
            ((-50, -100, -50), -32512),
            ((-50, -100, 0), -32512),
            ((-50, -100, 50), -32512),
            ((-50, -100, 100), -32512),
            ((-50, -50, -100), -32512),
            ((-50, -50, -50), -32512),
            ((-50, -50, 0), -32512),
            ((-50, -50, 50), -32512),
            ((-50, -50, 100), -32512),
            ((-50, 0, -100), -32512),
            ((-50, 0, -50), -32512),
            ((-50, 0, 0), -32512),
            ((-50, 0, 50), -32512),
            ((-50, 0, 100), -32512),
            ((-50, 50, -100), -32512),
            ((-50, 50, -50), -32512),
            ((-50, 50, 0), 57),
            ((-50, 50, 50), 57),
            ((-50, 50, 100), 60),
            ((-50, 100, -100), 80),
            ((-50, 100, -50), 0),
            ((-50, 100, 0), 0),
            ((-50, 100, 50), 0),
            ((-50, 100, 100), 0),
            ((0, -100, -100), -32512),
            ((0, -100, -50), -32512),
            ((0, -100, 0), -32512),
            ((0, -100, 50), -32512),
            ((0, -100, 100), -32512),
            ((0, -50, -100), -32512),
            ((0, -50, -50), -32512),
            ((0, -50, 0), -32512),
            ((0, -50, 50), -32512),
            ((0, -50, 100), -32512),
            ((0, 0, -100), -32512),
            ((0, 0, -50), -32512),
            ((0, 0, 0), -32512),
            ((0, 0, 50), -32512),
            ((0, 0, 100), -32512),
            ((0, 50, -100), -32512),
            ((0, 50, -50), -32512),
            ((0, 50, 0), 57),
            ((0, 50, 50), 60),
            ((0, 50, 100), 60),
            ((0, 100, -100), 0),
            ((0, 100, -50), 0),
            ((0, 100, 0), 0),
            ((0, 100, 50), 0),
            ((0, 100, 100), 0),
            ((50, -100, -100), -32512),
            ((50, -100, -50), -32512),
            ((50, -100, 0), -32512),
            ((50, -100, 50), -32512),
            ((50, -100, 100), -100),
            ((50, -50, -100), -32512),
            ((50, -50, -50), -32512),
            ((50, -50, 0), -32512),
            ((50, -50, 50), -32512),
            ((50, -50, 100), -60),
            ((50, 0, -100), -32512),
            ((50, 0, -50), -32512),
            ((50, 0, 0), -32512),
            ((50, 0, 50), -32512),
            ((50, 0, 100), -32512),
            ((50, 50, -100), -32512),
            ((50, 50, -50), -32512),
            ((50, 50, 0), 57),
            ((50, 50, 50), 60),
            ((50, 50, 100), 60),
            ((50, 100, -100), 0),
            ((50, 100, -50), 0),
            ((50, 100, 0), 0),
            ((50, 100, 50), 0),
            ((50, 100, 100), 0),
            ((100, -100, -100), -32512),
            ((100, -100, -50), -32512),
            ((100, -100, 0), -32512),
            ((100, -100, 50), -103),
            ((100, -100, 100), -100),
            ((100, -50, -100), -32512),
            ((100, -50, -50), -32512),
            ((100, -50, 0), -32512),
            ((100, -50, 50), -32512),
            ((100, -50, 100), -60),
            ((100, 0, -100), -32512),
            ((100, 0, -50), -32512),
            ((100, 0, 0), -32512),
            ((100, 0, 50), -32512),
            ((100, 0, 100), -32512),
            ((100, 50, -100), -32512),
            ((100, 50, -50), -32512),
            ((100, 50, 0), 57),
            ((100, 50, 50), 57),
            ((100, 50, 100), 60),
            ((100, 100, -100), 0),
            ((100, 100, -50), 0),
            ((100, 100, 0), 0),
            ((100, 100, 50), 0),
            ((100, 100, 100), 0),
        ];

        for ((x, y, z), result) in values {
            assert_eq!(
                aquifer.get_fluid_block_y(x, y, z, level.clone(), 80, true, &env),
                result
            );
        }

        let values = [
            ((-100, -100, -100), -32512),
            ((-100, -100, -50), -32512),
            ((-100, -100, 0), -32512),
            ((-100, -100, 50), -32512),
            ((-100, -100, 100), -32512),
            ((-100, -50, -100), -32512),
            ((-100, -50, -50), -63),
            ((-100, -50, 0), -63),
            ((-100, -50, 50), -63),
            ((-100, -50, 100), -32512),
            ((-100, 0, -100), -32512),
            ((-100, 0, -50), -32512),
            ((-100, 0, 0), -32512),
            ((-100, 0, 50), -32512),
            ((-100, 0, 100), -32512),
            ((-100, 50, -100), -32512),
            ((-100, 50, -50), -32512),
            ((-100, 50, 0), -32512),
            ((-100, 50, 50), -32512),
            ((-100, 50, 100), -32512),
            ((-100, 100, -100), -32512),
            ((-100, 100, -50), -32512),
            ((-100, 100, 0), -32512),
            ((-100, 100, 50), -32512),
            ((-100, 100, 100), -32512),
            ((-50, -100, -100), -32512),
            ((-50, -100, -50), -32512),
            ((-50, -100, 0), -32512),
            ((-50, -100, 50), -32512),
            ((-50, -100, 100), -32512),
            ((-50, -50, -100), -32512),
            ((-50, -50, -50), -32512),
            ((-50, -50, 0), -32512),
            ((-50, -50, 50), -32512),
            ((-50, -50, 100), -32512),
            ((-50, 0, -100), -32512),
            ((-50, 0, -50), -32512),
            ((-50, 0, 0), -32512),
            ((-50, 0, 50), -32512),
            ((-50, 0, 100), -32512),
            ((-50, 50, -100), -32512),
            ((-50, 50, -50), -32512),
            ((-50, 50, 0), -32512),
            ((-50, 50, 50), -32512),
            ((-50, 50, 100), -32512),
            ((-50, 100, -100), -32512),
            ((-50, 100, -50), -32512),
            ((-50, 100, 0), 80),
            ((-50, 100, 50), -32512),
            ((-50, 100, 100), -32512),
            ((0, -100, -100), -32512),
            ((0, -100, -50), -32512),
            ((0, -100, 0), -32512),
            ((0, -100, 50), -32512),
            ((0, -100, 100), -32512),
            ((0, -50, -100), -32512),
            ((0, -50, -50), -32512),
            ((0, -50, 0), -32512),
            ((0, -50, 50), -32512),
            ((0, -50, 100), -32512),
            ((0, 0, -100), -32512),
            ((0, 0, -50), -32512),
            ((0, 0, 0), -32512),
            ((0, 0, 50), -32512),
            ((0, 0, 100), -32512),
            ((0, 50, -100), -32512),
            ((0, 50, -50), -32512),
            ((0, 50, 0), -32512),
            ((0, 50, 50), -32512),
            ((0, 50, 100), -32512),
            ((0, 100, -100), -32512),
            ((0, 100, -50), -32512),
            ((0, 100, 0), 80),
            ((0, 100, 50), -32512),
            ((0, 100, 100), -32512),
            ((50, -100, -100), -32512),
            ((50, -100, -50), -32512),
            ((50, -100, 0), -32512),
            ((50, -100, 50), -32512),
            ((50, -100, 100), -100),
            ((50, -50, -100), -32512),
            ((50, -50, -50), -32512),
            ((50, -50, 0), -32512),
            ((50, -50, 50), -32512),
            ((50, -50, 100), -60),
            ((50, 0, -100), -32512),
            ((50, 0, -50), -32512),
            ((50, 0, 0), -32512),
            ((50, 0, 50), -32512),
            ((50, 0, 100), -32512),
            ((50, 50, -100), -32512),
            ((50, 50, -50), -32512),
            ((50, 50, 0), -32512),
            ((50, 50, 50), -32512),
            ((50, 50, 100), -32512),
            ((50, 100, -100), -32512),
            ((50, 100, -50), -32512),
            ((50, 100, 0), 80),
            ((50, 100, 50), -32512),
            ((50, 100, 100), -32512),
            ((100, -100, -100), -32512),
            ((100, -100, -50), -32512),
            ((100, -100, 0), -32512),
            ((100, -100, 50), -103),
            ((100, -100, 100), -100),
            ((100, -50, -100), -32512),
            ((100, -50, -50), -32512),
            ((100, -50, 0), -32512),
            ((100, -50, 50), -32512),
            ((100, -50, 100), -60),
            ((100, 0, -100), -32512),
            ((100, 0, -50), -32512),
            ((100, 0, 0), -32512),
            ((100, 0, 50), -32512),
            ((100, 0, 100), -32512),
            ((100, 50, -100), -32512),
            ((100, 50, -50), -32512),
            ((100, 50, 0), -32512),
            ((100, 50, 50), -32512),
            ((100, 50, 100), -32512),
            ((100, 100, -100), -32512),
            ((100, 100, -50), -32512),
            ((100, 100, 0), 80),
            ((100, 100, 50), -32512),
            ((100, 100, 100), -32512),
        ];

        for ((x, y, z), result) in values {
            assert_eq!(
                aquifer.get_fluid_block_y(x, y, z, level.clone(), 80, false, &env),
                result
            );
        }
    }

    #[test]
    fn test_get_fluid_level() {
        let (mut aquifer, mut funcs, env) = create_aquifer();
        let values = [
            ((-100, -100, -100), (-32512, *LAVA_BLOCK)),
            ((-100, -100, -50), (-32512, *LAVA_BLOCK)),
            ((-100, -100, 0), (-32512, *LAVA_BLOCK)),
            ((-100, -100, 50), (-32512, *LAVA_BLOCK)),
            ((-100, -100, 100), (-32512, *LAVA_BLOCK)),
            ((-100, -50, -100), (-32512, *WATER_BLOCK)),
            ((-100, -50, -50), (-63, *LAVA_BLOCK)),
            ((-100, -50, 0), (-63, *LAVA_BLOCK)),
            ((-100, -50, 50), (-63, *LAVA_BLOCK)),
            ((-100, -50, 100), (-32512, *WATER_BLOCK)),
            ((-100, 0, -100), (-32512, *WATER_BLOCK)),
            ((-100, 0, -50), (-32512, *WATER_BLOCK)),
            ((-100, 0, 0), (-32512, *WATER_BLOCK)),
            ((-100, 0, 50), (-32512, *WATER_BLOCK)),
            ((-100, 0, 100), (-32512, *WATER_BLOCK)),
            ((-100, 50, -100), (-32512, *WATER_BLOCK)),
            ((-100, 50, -50), (63, *WATER_BLOCK)),
            ((-100, 50, 0), (63, *WATER_BLOCK)),
            ((-100, 50, 50), (-32512, *WATER_BLOCK)),
            ((-100, 50, 100), (63, *WATER_BLOCK)),
            ((-100, 100, -100), (63, *WATER_BLOCK)),
            ((-100, 100, -50), (63, *WATER_BLOCK)),
            ((-100, 100, 0), (63, *WATER_BLOCK)),
            ((-100, 100, 50), (63, *WATER_BLOCK)),
            ((-100, 100, 100), (63, *WATER_BLOCK)),
            ((-50, -100, -100), (-32512, *LAVA_BLOCK)),
            ((-50, -100, -50), (-32512, *LAVA_BLOCK)),
            ((-50, -100, 0), (-32512, *LAVA_BLOCK)),
            ((-50, -100, 50), (-32512, *LAVA_BLOCK)),
            ((-50, -100, 100), (-32512, *LAVA_BLOCK)),
            ((-50, -50, -100), (-32512, *WATER_BLOCK)),
            ((-50, -50, -50), (-32512, *WATER_BLOCK)),
            ((-50, -50, 0), (-32512, *WATER_BLOCK)),
            ((-50, -50, 50), (-32512, *WATER_BLOCK)),
            ((-50, -50, 100), (-32512, *WATER_BLOCK)),
            ((-50, 0, -100), (-32512, *WATER_BLOCK)),
            ((-50, 0, -50), (-32512, *WATER_BLOCK)),
            ((-50, 0, 0), (-32512, *WATER_BLOCK)),
            ((-50, 0, 50), (-32512, *WATER_BLOCK)),
            ((-50, 0, 100), (-32512, *WATER_BLOCK)),
            ((-50, 50, -100), (-32512, *WATER_BLOCK)),
            ((-50, 50, -50), (63, *WATER_BLOCK)),
            ((-50, 50, 0), (63, *WATER_BLOCK)),
            ((-50, 50, 50), (-32512, *WATER_BLOCK)),
            ((-50, 50, 100), (63, *WATER_BLOCK)),
            ((-50, 100, -100), (63, *WATER_BLOCK)),
            ((-50, 100, -50), (63, *WATER_BLOCK)),
            ((-50, 100, 0), (63, *WATER_BLOCK)),
            ((-50, 100, 50), (63, *WATER_BLOCK)),
            ((-50, 100, 100), (63, *WATER_BLOCK)),
            ((0, -100, -100), (-32512, *LAVA_BLOCK)),
            ((0, -100, -50), (-32512, *LAVA_BLOCK)),
            ((0, -100, 0), (-32512, *LAVA_BLOCK)),
            ((0, -100, 50), (-32512, *LAVA_BLOCK)),
            ((0, -100, 100), (-32512, *LAVA_BLOCK)),
            ((0, -50, -100), (-32512, *WATER_BLOCK)),
            ((0, -50, -50), (-32512, *WATER_BLOCK)),
            ((0, -50, 0), (-32512, *WATER_BLOCK)),
            ((0, -50, 50), (-32512, *WATER_BLOCK)),
            ((0, -50, 100), (-32512, *WATER_BLOCK)),
            ((0, 0, -100), (-32512, *WATER_BLOCK)),
            ((0, 0, -50), (-32512, *WATER_BLOCK)),
            ((0, 0, 0), (-32512, *WATER_BLOCK)),
            ((0, 0, 50), (-32512, *WATER_BLOCK)),
            ((0, 0, 100), (-32512, *WATER_BLOCK)),
            ((0, 50, -100), (-32512, *WATER_BLOCK)),
            ((0, 50, -50), (63, *WATER_BLOCK)),
            ((0, 50, 0), (63, *WATER_BLOCK)),
            ((0, 50, 50), (-32512, *WATER_BLOCK)),
            ((0, 50, 100), (-32512, *WATER_BLOCK)),
            ((0, 100, -100), (63, *WATER_BLOCK)),
            ((0, 100, -50), (63, *WATER_BLOCK)),
            ((0, 100, 0), (63, *WATER_BLOCK)),
            ((0, 100, 50), (63, *WATER_BLOCK)),
            ((0, 100, 100), (63, *WATER_BLOCK)),
            ((50, -100, -100), (-32512, *LAVA_BLOCK)),
            ((50, -100, -50), (-32512, *LAVA_BLOCK)),
            ((50, -100, 0), (-32512, *LAVA_BLOCK)),
            ((50, -100, 50), (-32512, *LAVA_BLOCK)),
            ((50, -100, 100), (-100, *LAVA_BLOCK)),
            ((50, -50, -100), (-32512, *WATER_BLOCK)),
            ((50, -50, -50), (-32512, *WATER_BLOCK)),
            ((50, -50, 0), (-32512, *WATER_BLOCK)),
            ((50, -50, 50), (-32512, *WATER_BLOCK)),
            ((50, -50, 100), (-60, *WATER_BLOCK)),
            ((50, 0, -100), (-32512, *WATER_BLOCK)),
            ((50, 0, -50), (-32512, *WATER_BLOCK)),
            ((50, 0, 0), (-32512, *WATER_BLOCK)),
            ((50, 0, 50), (-32512, *WATER_BLOCK)),
            ((50, 0, 100), (-32512, *WATER_BLOCK)),
            ((50, 50, -100), (-32512, *WATER_BLOCK)),
            ((50, 50, -50), (63, *WATER_BLOCK)),
            ((50, 50, 0), (63, *WATER_BLOCK)),
            ((50, 50, 50), (63, *WATER_BLOCK)),
            ((50, 50, 100), (-32512, *WATER_BLOCK)),
            ((50, 100, -100), (-32512, *WATER_BLOCK)),
            ((50, 100, -50), (63, *WATER_BLOCK)),
            ((50, 100, 0), (63, *WATER_BLOCK)),
            ((50, 100, 50), (63, *WATER_BLOCK)),
            ((50, 100, 100), (63, *WATER_BLOCK)),
            ((100, -100, -100), (-32512, *LAVA_BLOCK)),
            ((100, -100, -50), (-32512, *LAVA_BLOCK)),
            ((100, -100, 0), (-32512, *LAVA_BLOCK)),
            ((100, -100, 50), (-103, *LAVA_BLOCK)),
            ((100, -100, 100), (-100, *LAVA_BLOCK)),
            ((100, -50, -100), (-32512, *WATER_BLOCK)),
            ((100, -50, -50), (-32512, *WATER_BLOCK)),
            ((100, -50, 0), (-32512, *WATER_BLOCK)),
            ((100, -50, 50), (-32512, *WATER_BLOCK)),
            ((100, -50, 100), (-60, *LAVA_BLOCK)),
            ((100, 0, -100), (-32512, *WATER_BLOCK)),
            ((100, 0, -50), (-32512, *WATER_BLOCK)),
            ((100, 0, 0), (-32512, *WATER_BLOCK)),
            ((100, 0, 50), (-32512, *WATER_BLOCK)),
            ((100, 0, 100), (-32512, *WATER_BLOCK)),
            ((100, 50, -100), (63, *WATER_BLOCK)),
            ((100, 50, -50), (63, *WATER_BLOCK)),
            ((100, 50, 0), (63, *WATER_BLOCK)),
            ((100, 50, 50), (63, *WATER_BLOCK)),
            ((100, 50, 100), (-32512, *WATER_BLOCK)),
            ((100, 100, -100), (63, *WATER_BLOCK)),
            ((100, 100, -50), (63, *WATER_BLOCK)),
            ((100, 100, 0), (63, *WATER_BLOCK)),
            ((100, 100, 50), (63, *WATER_BLOCK)),
            ((100, 100, 100), (63, *WATER_BLOCK)),
        ];

        for ((x, y, z), (y1, state)) in values {
            let level = aquifer.get_fluid_level(x, y, z, &mut funcs, &env);
            assert_eq!(level.max_y, y1);
            assert_eq!(level.state, state);
        }
    }

    #[test]
    fn test_calculate_density() {
        let (mut aquifer, _, env) = create_aquifer();

        let values = [
            ((-100, -100, -100, 0, 0), 0.0),
            ((-100, -100, -50, 50, 0), -19.3),
            ((-100, -100, 0, 0, 0), 0.0),
            ((-100, -100, 50, 50, 0), -19.3),
            ((-100, -100, 100, 0, 0), 0.0),
            ((-100, -50, -100, 0, 0), 0.0),
            ((-100, -50, -50, 50, 0), -9.3),
            ((-100, -50, 0, 0, 0), 0.0),
            ((-100, -50, 50, 50, 0), -9.3),
            ((-100, -50, 100, 0, 0), 0.0),
            ((-100, 0, -100, 0, 0), 0.0),
            ((-100, 0, -50, -50, 0), 0.2083850667904572),
            ((-100, 0, 0, 0, 0), 0.0),
            ((-100, 0, 50, 50, 0), 2.069189235272414),
            ((-100, 0, 100, 0, 0), 0.0),
            ((-100, 50, -100, 0, 0), 0.0),
            ((-100, 50, -50, -50, 0), -40.4),
            ((-100, 50, 0, 0, 0), 0.0),
            ((-100, 50, 50, -50, 0), -40.4),
            ((-100, 50, 100, 0, 0), 0.0),
            ((-100, 100, -100, 0, 0), 0.0),
            ((-100, 100, -50, -50, 0), -80.4),
            ((-100, 100, 0, 0, 0), 0.0),
            ((-100, 100, 50, -50, 0), -80.4),
            ((-100, 100, 100, 0, 0), 0.0),
            ((-50, -100, -100, 0, 50), -19.3),
            ((-50, -100, -50, 50, 50), 0.0),
            ((-50, -100, 0, 0, -50), -9.3),
            ((-50, -100, 50, 50, -50), -9.3),
            ((-50, -100, 100, 0, -50), -9.3),
            ((-50, -50, -100, 0, 50), -9.3),
            ((-50, -50, -50, 50, 50), 0.0),
            ((-50, -50, 0, 0, -50), 2.2042949518442185),
            ((-50, -50, 50, 50, -50), 1.8767275908406176),
            ((-50, -50, 100, 0, -50), 2.3399656359995133),
            ((-50, 0, -100, 0, -50), -0.08405949841069171),
            ((-50, 0, -50, -50, -50), 0.0),
            ((-50, 0, 0, 0, -50), 0.3902410585192353),
            ((-50, 0, 50, 50, -50), 66.0),
            ((-50, 0, 100, 0, -50), -0.7930165090675787),
            ((-50, 50, -100, 0, -50), -40.4),
            ((-50, 50, -50, -50, -50), 0.0),
            ((-50, 50, 0, 0, -50), -40.4),
            ((-50, 50, 50, -50, 50), -0.35570822400215646),
            ((-50, 50, 100, 0, 50), -0.16770224497207317),
            ((-50, 100, -100, 0, -50), -80.4),
            ((-50, 100, -50, -50, -50), 0.0),
            ((-50, 100, 0, 0, -50), -80.4),
            ((-50, 100, 50, -50, 50), -40.4),
            ((-50, 100, 100, 0, 50), -40.4),
            ((0, -100, -100, 0, 0), 0.0),
            ((0, -100, -50, -50, 0), -9.3),
            ((0, -100, 0, 0, 0), 0.0),
            ((0, -100, 50, 50, 0), -19.3),
            ((0, -100, 100, 0, 0), 0.0),
            ((0, -50, -100, 0, 0), 0.0),
            ((0, -50, -50, -50, 0), 2.857141340264507),
            ((0, -50, 0, 0, 0), 0.0),
            ((0, -50, 50, 50, 0), -9.3),
            ((0, -50, 100, 0, 0), 0.0),
            ((0, 0, -100, 0, 0), 0.0),
            ((0, 0, -50, -50, 0), -0.1361016501707068),
            ((0, 0, 0, 0, 0), 0.0),
            ((0, 0, 50, 50, 0), 1.9841279541408636),
            ((0, 0, 100, 0, 0), 0.0),
            ((0, 50, -100, 0, 0), 0.0),
            ((0, 50, -50, -50, 0), -40.4),
            ((0, 50, 0, 0, 0), 0.0),
            ((0, 50, 50, 50, 0), -0.36331007530382964),
            ((0, 50, 100, 0, 0), 0.0),
            ((0, 100, -100, 0, 0), 0.0),
            ((0, 100, -50, -50, 0), -80.4),
            ((0, 100, 0, 0, 0), 0.0),
            ((0, 100, 50, 50, 0), -40.4),
            ((0, 100, 100, 0, 0), 0.0),
            ((50, -100, -100, 0, 50), -19.3),
            ((50, -100, -50, -50, 50), -9.3),
            ((50, -100, 0, 0, 50), -19.3),
            ((50, -100, 50, -50, -50), 0.0),
            ((50, -100, 100, 0, -50), -9.3),
            ((50, -50, -100, 0, 50), -9.3),
            ((50, -50, -50, -50, 50), 1.619242225388449),
            ((50, -50, 0, 0, 50), -9.3),
            ((50, -50, 50, -50, -50), 0.0),
            ((50, -50, 100, 0, -50), 2.1561171703198188),
            ((50, 0, -100, 0, 50), 2.6298865590685954),
            ((50, 0, -50, -50, 50), 66.0),
            ((50, 0, 0, 0, 50), 2.572198917833846),
            ((50, 0, 50, 50, 50), 0.0),
            ((50, 0, 100, 0, 50), 2.082884998883258),
            ((50, 50, -100, 0, -50), -40.4),
            ((50, 50, -50, 50, -50), -0.1894344852785401),
            ((50, 50, 0, 0, 50), -0.7155260733519367),
            ((50, 50, 50, 50, 50), 0.0),
            ((50, 50, 100, 0, 50), -0.4132183490530098),
            ((50, 100, -100, 0, -50), -80.4),
            ((50, 100, -50, 50, -50), -40.4),
            ((50, 100, 0, 0, 50), -40.4),
            ((50, 100, 50, 50, 50), 0.0),
            ((50, 100, 100, 0, 50), -40.4),
            ((100, -100, -100, 0, 0), 0.0),
            ((100, -100, -50, -50, 0), -9.3),
            ((100, -100, 0, 0, 0), 0.0),
            ((100, -100, 50, -50, 0), -9.3),
            ((100, -100, 100, 0, 0), 0.0),
            ((100, -50, -100, 0, 0), 0.0),
            ((100, -50, -50, -50, 0), 1.6711026207576742),
            ((100, -50, 0, 0, 0), 0.0),
            ((100, -50, 50, -50, 0), 2.042353012197518),
            ((100, -50, 100, 0, 0), 0.0),
            ((100, 0, -100, 0, 0), 0.0),
            ((100, 0, -50, -50, 0), 0.3145492757856567),
            ((100, 0, 0, 0, 0), 0.0),
            ((100, 0, 50, 50, 0), 2.27260703684609),
            ((100, 0, 100, 0, 0), 0.0),
            ((100, 50, -100, 0, 0), 0.0),
            ((100, 50, -50, 50, 0), -0.16949328993376553),
            ((100, 50, 0, 0, 0), 0.0),
            ((100, 50, 50, 50, 0), 0.5196380801381327),
            ((100, 50, 100, 0, 0), 0.0),
            ((100, 100, -100, 0, 0), 0.0),
            ((100, 100, -50, 50, 0), -40.4),
            ((100, 100, 0, 0, 0), 0.0),
            ((100, 100, 50, 50, 0), -40.4),
            ((100, 100, 100, 0, 0), 0.0),
        ];

        for ((x, y, z, h1, h2), result) in values {
            let level1 = FluidLevel::new(h1, *WATER_BLOCK);
            let level2 = FluidLevel::new(h2, *WATER_BLOCK);
            let pos = &NoisePos::Unblended(UnblendedNoisePos::new(x, y, z));
            let sample = aquifer.barrier_noise.sample_mut(pos, &env);
            assert_eq!(
                aquifer.calculate_density(pos, sample, level1, level2),
                result
            );
        }
    }

    #[test]
    fn test_apply() {
        let (mut aquifer, mut funcs, env) = create_aquifer();
        let values = [
            ((112, -100, 64, 0.037482421875), None),
            ((112, -100, 66, 0.037482421875), None),
            ((112, -100, 68, 0.037482421875), None),
            ((112, -100, 70, 0.037482421875), None),
            ((112, -100, 72, 0.037482421875), None),
            ((112, -100, 74, 0.037482421875), None),
            ((112, -80, 64, 0.037482421875), None),
            ((112, -80, 66, 0.037482421875), None),
            ((112, -80, 68, 0.037482421875), None),
            ((112, -80, 70, 0.037482421875), None),
            ((112, -80, 72, 0.037482421875), None),
            ((112, -80, 74, 0.037482421875), None),
            ((112, -60, 64, 0.04861063447117113), None),
            ((112, -60, 66, 0.04924175767418443), None),
            ((112, -60, 68, 0.04989611436947345), None),
            ((112, -60, 70, 0.0505756649980066), None),
            ((112, -60, 72, 0.051280297037227154), None),
            ((112, -60, 74, 0.05200473277817172), None),
            ((112, -40, 64, 0.10768778489063564), None),
            ((112, -40, 66, 0.11167582884587932), None),
            ((112, -40, 68, 0.1150242394949102), None),
            ((112, -40, 70, 0.11846243756482717), None),
            ((112, -40, 72, 0.12198087391634192), None),
            ((112, -40, 74, 0.12558796524209848), None),
            ((112, -20, 64, 0.10797485850904144), None),
            ((112, -20, 66, 0.10725848123542484), None),
            ((112, -20, 68, 0.1061187765379769), None),
            ((112, -20, 70, 0.10630527141203541), None),
            ((112, -20, 72, 0.10955530075188523), None),
            ((112, -20, 74, 0.11296050502041746), None),
            ((112, 0, 64, -0.0016242211141160837), None),
            ((112, 0, 66, -1.8270827225992477E-4), None),
            ((112, 0, 68, 0.0013878562706830444), None),
            ((112, 0, 70, 0.0030297756784707407), None),
            ((112, 0, 72, 0.004722310162617811), None),
            ((112, 0, 74, 0.0064699315064971315), None),
            ((112, 20, 64, 0.042774410930765096), None),
            ((112, 20, 66, 0.040191327400772414), None),
            ((112, 20, 68, 0.0375637494138927), None),
            ((112, 20, 70, 0.03485919313914573), None),
            ((112, 20, 72, 0.032058901444915404), None),
            ((112, 20, 74, 0.029170651813548235), None),
            ((112, 40, 64, 0.011596925999172775), None),
            ((112, 40, 66, 0.014957019593043965), None),
            ((112, 40, 68, 0.01827627643898028), None),
            ((112, 40, 70, 0.021477214738188897), None),
            ((112, 40, 72, 0.02448508577325224), None),
            ((112, 40, 74, 0.027262647782191486), None),
            ((112, 60, 64, 0.14338446306313316), None),
            ((112, 60, 66, 0.16772904485726645), None),
            ((112, 60, 68, 0.1756309873589998), None),
            ((112, 60, 70, 0.1782686032102433), None),
            ((112, 60, 72, 0.18822148746793055), None),
            ((112, 60, 74, 0.20387997189913717), None),
            ((112, 80, 64, -0.28931054817132484), Some(BlockState::AIR)),
            ((112, 80, 66, -0.2808098154769529), Some(BlockState::AIR)),
            ((112, 80, 68, -0.2806908647477032), Some(BlockState::AIR)),
            ((112, 80, 70, -0.28068300576359284), Some(BlockState::AIR)),
            ((112, 80, 72, -0.2805878392398348), Some(BlockState::AIR)),
            ((112, 80, 74, -0.27824504138444317), Some(BlockState::AIR)),
            ((112, 100, 64, -0.4583333333333333), Some(BlockState::AIR)),
            ((112, 100, 66, -0.4583333333333333), Some(BlockState::AIR)),
            ((112, 100, 68, -0.4583333333333333), Some(BlockState::AIR)),
            ((112, 100, 70, -0.4583333333333333), Some(BlockState::AIR)),
            ((112, 100, 72, -0.4583333333333333), Some(BlockState::AIR)),
            ((112, 100, 74, -0.4583333333333333), Some(BlockState::AIR)),
            ((114, -100, 64, 0.037482421875), None),
            ((114, -100, 66, 0.037482421875), None),
            ((114, -100, 68, 0.037482421875), None),
            ((114, -100, 70, 0.037482421875), None),
            ((114, -100, 72, 0.037482421875), None),
            ((114, -100, 74, 0.037482421875), None),
            ((114, -80, 64, 0.037482421875), None),
            ((114, -80, 66, 0.037482421875), None),
            ((114, -80, 68, 0.037482421875), None),
            ((114, -80, 70, 0.037482421875), None),
            ((114, -80, 72, 0.037482421875), None),
            ((114, -80, 74, 0.037482421875), None),
            ((114, -60, 64, 0.0491033911468506), None),
            ((114, -60, 66, 0.04974948234981454), None),
            ((114, -60, 68, 0.05041864489941496), None),
            ((114, -60, 70, 0.051112999802414315), None),
            ((114, -60, 72, 0.051832684146704056), None),
            ((114, -60, 74, 0.05257231446254198), None),
            ((114, -40, 64, 0.1122919611041932), None),
            ((114, -40, 66, 0.11636247433302638), None),
            ((114, -40, 68, 0.11979640774332366), None),
            ((114, -40, 70, 0.12330055500715384), None),
            ((114, -40, 72, 0.1268619579414046), None),
            ((114, -40, 74, 0.13048474950215), None),
            ((114, -20, 64, 0.10435296791344484), None),
            ((114, -20, 66, 0.10558672714675042), None),
            ((114, -20, 68, 0.10831868013423275), None),
            ((114, -20, 70, 0.11121282163190734), None),
            ((114, -20, 72, 0.11433776346079558), None),
            ((114, -20, 74, 0.11770444723497474), None),
            ((114, 0, 64, -0.0026209759846139574), Some(*WATER_BLOCK)),
            ((114, 0, 66, -0.0011869543056835608), Some(*WATER_BLOCK)),
            ((114, 0, 68, 3.9347454816496854E-4), None),
            ((114, 0, 70, 0.002068623223791626), None),
            ((114, 0, 72, 0.0038193250297024243), None),
            ((114, 0, 74, 0.005646396361630968), None),
            ((114, 20, 64, 0.04183884597405475), None),
            ((114, 20, 66, 0.039334239558802865), None),
            ((114, 20, 68, 0.0367933735261107), None),
            ((114, 20, 70, 0.03417828296442246), None),
            ((114, 20, 72, 0.03146119496538579), None),
            ((114, 20, 74, 0.028640178813057193), None),
            ((114, 40, 64, 0.0011727557986376189), None),
            ((114, 40, 66, 0.004478197250912497), None),
            ((114, 40, 68, 0.007807656734373648), None),
            ((114, 40, 70, 0.0110982528501824), None),
            ((114, 40, 72, 0.014287968294347897), None),
            ((114, 40, 74, 0.017340060385442255), None),
            ((114, 60, 64, 0.057307692379898946), None),
            ((114, 60, 66, 0.07909878599088975), None),
            ((114, 60, 68, 0.09103769264897049), None),
            ((114, 60, 70, 0.10529675531043547), None),
            ((114, 60, 72, 0.1261191394093652), None),
            ((114, 60, 74, 0.15323465023530602), None),
            ((114, 80, 64, -0.3135251519473628), Some(BlockState::AIR)),
            ((114, 80, 66, -0.3092766951165722), Some(BlockState::AIR)),
            ((114, 80, 68, -0.3063751991759311), Some(BlockState::AIR)),
            ((114, 80, 70, -0.3004342091280733), Some(BlockState::AIR)),
            ((114, 80, 72, -0.29703745590700253), Some(BlockState::AIR)),
            ((114, 80, 74, -0.2920638815250855), Some(BlockState::AIR)),
            ((114, 100, 64, -0.4583333333333333), Some(BlockState::AIR)),
            ((114, 100, 66, -0.4583333333333333), Some(BlockState::AIR)),
            ((114, 100, 68, -0.4583333333333333), Some(BlockState::AIR)),
            ((114, 100, 70, -0.4583333333333333), Some(BlockState::AIR)),
            ((114, 100, 72, -0.4583333333333333), Some(BlockState::AIR)),
            ((114, 100, 74, -0.4583333333333333), Some(BlockState::AIR)),
            ((116, -100, 64, 0.037482421875), None),
            ((116, -100, 66, 0.037482421875), None),
            ((116, -100, 68, 0.037482421875), None),
            ((116, -100, 70, 0.037482421875), None),
            ((116, -100, 72, 0.037482421875), None),
            ((116, -100, 74, 0.037482421875), None),
            ((116, -80, 64, 0.037482421875), None),
            ((116, -80, 66, 0.037482421875), None),
            ((116, -80, 68, 0.037482421875), None),
            ((116, -80, 70, 0.037482421875), None),
            ((116, -80, 72, 0.037482421875), None),
            ((116, -80, 74, 0.037482421875), None),
            ((116, -60, 64, 0.049510031628008565), None),
            ((116, -60, 66, 0.0501688864114582), None),
            ((116, -60, 68, 0.05085081029508635), None),
            ((116, -60, 70, 0.051557962654552196), None),
            ((116, -60, 72, 0.05229073376331405), None),
            ((116, -60, 74, 0.05304386684600273), None),
            ((116, -40, 64, 0.11647696380354154), None),
            ((116, -40, 66, 0.12065670598690036), None),
            ((116, -40, 68, 0.124186198423662), None),
            ((116, -40, 70, 0.12776934001653206), None),
            ((116, -40, 72, 0.1313900411197126), None),
            ((116, -40, 74, 0.13504793394752057), None),
            ((116, -20, 64, 0.10797310440989095), None),
            ((116, -20, 66, 0.11052803071968675), None),
            ((116, -20, 68, 0.11313075983384659), None),
            ((116, -20, 70, 0.11589560860382501), None),
            ((116, -20, 72, 0.11889599517563405), None),
            ((116, -20, 74, 0.12214992807094607), None),
            ((116, 0, 64, -0.003764380972543319), Some(*WATER_BLOCK)),
            ((116, 0, 66, -0.002339168705169207), Some(*WATER_BLOCK)),
            ((116, 0, 68, -7.530784033722614E-4), Some(*WATER_BLOCK)),
            ((116, 0, 70, 9.517226455286942E-4), None),
            ((116, 0, 72, 0.0027605740566328273), None),
            ((116, 0, 74, 0.0046712919320928475), None),
            ((116, 20, 64, 0.04027812111674997), None),
            ((116, 20, 66, 0.03846080745866824), None),
            ((116, 20, 68, 0.03602535830962611), None),
            ((116, 20, 70, 0.033477031827460015), None),
            ((116, 20, 72, 0.03082854239914883), None),
            ((116, 20, 74, 0.028967404819271153), None),
            ((116, 40, 64, -0.009355588931802767), None),
            ((116, 40, 66, -0.006094366713842806), Some(BlockState::AIR)),
            ((116, 40, 68, -0.0027537988904787606), Some(BlockState::AIR)),
            ((116, 40, 70, 6.165942717199293E-4), None),
            ((116, 40, 72, 0.00396682662711753), None),
            ((116, 40, 74, 0.007260950530417173), None),
            ((116, 60, 64, 0.022624582978071506), None),
            ((116, 60, 66, 0.023942871216082746), None),
            ((116, 60, 68, 0.037192323513527165), None),
            ((116, 60, 70, 0.053305618429865455), None),
            ((116, 60, 72, 0.06694547220363958), None),
            ((116, 60, 74, 0.08711813973093903), None),
            ((116, 80, 64, -0.3326652310213258), Some(BlockState::AIR)),
            ((116, 80, 66, -0.32962834810938174), Some(BlockState::AIR)),
            ((116, 80, 68, -0.32236370014057947), Some(BlockState::AIR)),
            ((116, 80, 70, -0.31670491006554574), Some(BlockState::AIR)),
            ((116, 80, 72, -0.3130639601887072), Some(BlockState::AIR)),
            ((116, 80, 74, -0.3124769234268471), Some(BlockState::AIR)),
            ((116, 100, 64, -0.4583333333333333), Some(BlockState::AIR)),
            ((116, 100, 66, -0.4583333333333333), Some(BlockState::AIR)),
            ((116, 100, 68, -0.4583333333333333), Some(BlockState::AIR)),
            ((116, 100, 70, -0.4583333333333333), Some(BlockState::AIR)),
            ((116, 100, 72, -0.4583333333333333), Some(BlockState::AIR)),
            ((116, 100, 74, -0.4583333333333333), Some(BlockState::AIR)),
            ((118, -100, 64, 0.037482421875), None),
            ((118, -100, 66, 0.037482421875), None),
            ((118, -100, 68, 0.037482421875), None),
            ((118, -100, 70, 0.037482421875), None),
            ((118, -100, 72, 0.037482421875), None),
            ((118, -100, 74, 0.037482421875), None),
            ((118, -80, 64, 0.037482421875), None),
            ((118, -80, 66, 0.037482421875), None),
            ((118, -80, 68, 0.037482421875), None),
            ((118, -80, 70, 0.037482421875), None),
            ((118, -80, 72, 0.037482421875), None),
            ((118, -80, 74, 0.037482421875), None),
            ((118, -60, 64, 0.04983026397203373), None),
            ((118, -60, 66, 0.05049926028532013), None),
            ((118, -60, 68, 0.05119129093033625), None),
            ((118, -60, 70, 0.05190836104462707), None),
            ((118, -60, 72, 0.0526510917415344), None),
            ((118, -60, 74, 0.05341462434689053), None),
            ((118, -40, 64, 0.12023506214440967), None),
            ((118, -40, 66, 0.12453642919562771), None),
            ((118, -40, 68, 0.1281709824736104), None),
            ((118, -40, 70, 0.13184553502467083), None),
            ((118, -40, 72, 0.13554113610267055), None),
            ((118, -40, 74, 0.13925273367632113), None),
            ((118, -20, 64, 0.11282016461369758), None),
            ((118, -20, 66, 0.11526093589883914), None),
            ((118, -20, 68, 0.11774539644635261), None),
            ((118, -20, 70, 0.12038440430256446), None),
            ((118, -20, 72, 0.12325317705242089), None),
            ((118, -20, 74, 0.12637678353248477), None),
            ((118, 0, 64, -0.00501589634392619), Some(*WATER_BLOCK)),
            ((118, 0, 66, -0.003601631485605401), Some(*WATER_BLOCK)),
            ((118, 0, 68, -0.0020166185756455924), Some(*WATER_BLOCK)),
            ((118, 0, 70, -2.901294172670075E-4), Some(*WATER_BLOCK)),
            ((118, 0, 72, 0.0015704124446037308), None),
            ((118, 0, 74, 0.0035605198020311826), None),
            ((118, 20, 64, 0.040232966220497414), None),
            ((118, 20, 66, 0.039486572716430204), None),
            ((118, 20, 68, 0.03885242305023035), None),
            ((118, 20, 70, 0.03837601204655802), None),
            ((118, 20, 72, 0.03770430407795084), None),
            ((118, 20, 74, 0.03707763999605109), None),
            ((118, 40, 64, -0.016298811685686653), None),
            ((118, 40, 66, -0.016656636719901533), Some(BlockState::AIR)),
            ((118, 40, 68, -0.01330299024830442), Some(BlockState::AIR)),
            ((118, 40, 70, -0.009864486324034218), Some(BlockState::AIR)),
            ((118, 40, 72, -0.006380723268648157), Some(BlockState::AIR)),
            ((118, 40, 74, -0.002886835272463701), Some(BlockState::AIR)),
            ((118, 60, 64, 0.006086790713922152), None),
            ((118, 60, 66, 0.006014479808113486), None),
            ((118, 60, 68, 0.008953321476532182), None),
            ((118, 60, 70, 0.011915636473415587), None),
            ((118, 60, 72, 0.01001192490238903), None),
            ((118, 60, 74, 0.0075500927486281426), None),
            ((118, 80, 64, -0.3462118919745469), Some(BlockState::AIR)),
            ((118, 80, 66, -0.34419241078645835), Some(BlockState::AIR)),
            ((118, 80, 68, -0.33580861045450133), Some(BlockState::AIR)),
            ((118, 80, 70, -0.33008534054566163), Some(BlockState::AIR)),
            ((118, 80, 72, -0.333649815109498), Some(BlockState::AIR)),
            ((118, 80, 74, -0.33771329428807284), Some(BlockState::AIR)),
            ((118, 100, 64, -0.4583333333333333), Some(BlockState::AIR)),
            ((118, 100, 66, -0.4583333333333333), Some(BlockState::AIR)),
            ((118, 100, 68, -0.4583333333333333), Some(BlockState::AIR)),
            ((118, 100, 70, -0.4583333333333333), Some(BlockState::AIR)),
            ((118, 100, 72, -0.4583333333333333), Some(BlockState::AIR)),
            ((118, 100, 74, -0.4583333333333333), Some(BlockState::AIR)),
            ((120, -100, 64, 0.037482421875), None),
            ((120, -100, 66, 0.037482421875), None),
            ((120, -100, 68, 0.037482421875), None),
            ((120, -100, 70, 0.037482421875), None),
            ((120, -100, 72, 0.037482421875), None),
            ((120, -100, 74, 0.037482421875), None),
            ((120, -80, 64, 0.037482421875), None),
            ((120, -80, 66, 0.037482421875), None),
            ((120, -80, 68, 0.037482421875), None),
            ((120, -80, 70, 0.037482421875), None),
            ((120, -80, 72, 0.037482421875), None),
            ((120, -80, 74, 0.037482421875), None),
            ((120, -60, 64, 0.05006400032487905), None),
            ((120, -60, 66, 0.05074065517392799), None),
            ((120, -60, 68, 0.05144011744518559), None),
            ((120, -60, 70, 0.05216399757431933), None),
            ((120, -60, 72, 0.052913091382129525), None),
            ((120, -60, 74, 0.053683198493088384), None),
            ((120, -40, 64, 0.1235607292468892), None),
            ((120, -40, 66, 0.12798336289135304), None),
            ((120, -40, 68, 0.13173164094126777), None),
            ((120, -40, 70, 0.13550880522456066), None),
            ((120, -40, 72, 0.1392932761010718), None),
            ((120, -40, 74, 0.14307520910884483), None),
            ((120, -20, 64, 0.11746888767713162), None),
            ((120, -20, 66, 0.1198086187834423), None),
            ((120, -20, 68, 0.12218509691580316), None),
            ((120, -20, 70, 0.12469970986670785), None),
            ((120, -20, 72, 0.1274266324562779), None),
            ((120, -20, 74, 0.13039812171785095), None),
            ((120, 0, 64, -0.006329576321547214), Some(*WATER_BLOCK)),
            ((120, 0, 66, -0.004930192503238298), Some(*WATER_BLOCK)),
            ((120, 0, 68, -0.003355964670343278), Some(*WATER_BLOCK)),
            ((120, 0, 70, -0.0016206542469077872), Some(*WATER_BLOCK)),
            ((120, 0, 72, 2.77535008128212E-4), None),
            ((120, 0, 74, 0.002332419553726638), None),
            ((120, 20, 64, 0.04783863627983937), None),
            ((120, 20, 66, 0.04699098011102891), None),
            ((120, 20, 68, 0.046231913852781324), None),
            ((120, 20, 70, 0.04560004002476374), None),
            ((120, 20, 72, 0.04512419653798615), None),
            ((120, 20, 74, 0.04482086018104904), None),
            ((120, 40, 64, -0.017456523705167773), None),
            ((120, 40, 66, -0.020044623482270124), Some(BlockState::AIR)),
            ((120, 40, 68, -0.022372181411266172), Some(BlockState::AIR)),
            ((120, 40, 70, -0.020228945291907708), Some(BlockState::AIR)),
            ((120, 40, 72, -0.01664436674077766), Some(BlockState::AIR)),
            ((120, 40, 74, -0.013001583733654043), Some(BlockState::AIR)),
            ((120, 60, 64, -0.010805185122555435), Some(*WATER_BLOCK)),
            ((120, 60, 66, -0.011684313707812422), Some(*WATER_BLOCK)),
            ((120, 60, 68, -0.007705484690135335), Some(*WATER_BLOCK)),
            ((120, 60, 70, -0.012326309226980426), Some(*WATER_BLOCK)),
            ((120, 60, 72, -0.019043795741958334), Some(*WATER_BLOCK)),
            ((120, 60, 74, -0.023185441889689514), Some(*WATER_BLOCK)),
            ((120, 80, 64, -0.3611328625547435), Some(BlockState::AIR)),
            ((120, 80, 66, -0.3586517592327399), Some(BlockState::AIR)),
            ((120, 80, 68, -0.3524534485283812), Some(BlockState::AIR)),
            ((120, 80, 70, -0.35323218454039057), Some(BlockState::AIR)),
            ((120, 80, 72, -0.36213549677301105), Some(BlockState::AIR)),
            ((120, 80, 74, -0.3684474143996314), Some(BlockState::AIR)),
            ((120, 100, 64, -0.4583333333333333), Some(BlockState::AIR)),
            ((120, 100, 66, -0.4583333333333333), Some(BlockState::AIR)),
            ((120, 100, 68, -0.4583333333333333), Some(BlockState::AIR)),
            ((120, 100, 70, -0.4583333333333333), Some(BlockState::AIR)),
            ((120, 100, 72, -0.4583333333333333), Some(BlockState::AIR)),
            ((120, 100, 74, -0.4583333333333333), Some(BlockState::AIR)),
            ((122, -100, 64, 0.037482421875), None),
            ((122, -100, 66, 0.037482421875), None),
            ((122, -100, 68, 0.037482421875), None),
            ((122, -100, 70, 0.037482421875), None),
            ((122, -100, 72, 0.037482421875), None),
            ((122, -100, 74, 0.037482421875), None),
            ((122, -80, 64, 0.037482421875), None),
            ((122, -80, 66, 0.037482421875), None),
            ((122, -80, 68, 0.037482421875), None),
            ((122, -80, 70, 0.037482421875), None),
            ((122, -80, 72, 0.037482421875), None),
            ((122, -80, 74, 0.037482421875), None),
            ((122, -60, 64, 0.050211152183993704), None),
            ((122, -60, 66, 0.05089355899866971), None),
            ((122, -60, 68, 0.05159826596516877), None),
            ((122, -60, 70, 0.05232622949491639), None),
            ((122, -60, 72, 0.053078329171471185), None),
            ((122, -60, 74, 0.05385122904837061), None),
            ((122, -40, 64, 0.12644943947294945), None),
            ((122, -40, 66, 0.13095960991588632), None),
            ((122, -40, 68, 0.13485451601704876), None),
            ((122, -40, 70, 0.13874405974643175), None),
            ((122, -40, 72, 0.1426292409194739), None),
            ((122, -40, 74, 0.14649538659711875), None),
            ((122, -20, 64, 0.12191638116406471), None),
            ((122, -20, 66, 0.12416901766463509), None),
            ((122, -20, 68, 0.1264480340172126), None),
            ((122, -20, 70, 0.12884021810202248), None),
            ((122, -20, 72, 0.13141636494496137), None),
            ((122, -20, 74, 0.13421609155559988), None),
            ((122, 0, 64, -0.007667256303541582), Some(*WATER_BLOCK)),
            ((122, 0, 66, -0.006288822820341533), Some(*WATER_BLOCK)),
            ((122, 0, 68, -0.004737470102527975), Some(*WATER_BLOCK)),
            ((122, 0, 70, -0.0030099389619020873), Some(*WATER_BLOCK)),
            ((122, 0, 72, -0.0010942861750551764), Some(*WATER_BLOCK)),
            ((122, 0, 74, 0.001001992078975999), None),
            ((122, 20, 64, 0.05535892661135848), None),
            ((122, 20, 66, 0.05444413322973901), None),
            ((122, 20, 68, 0.05359719969130291), None),
            ((122, 20, 70, 0.05284624631887433), None),
            ((122, 20, 72, 0.052209127741325245), None),
            ((122, 20, 74, 0.05170112238940831), None),
            ((122, 40, 64, -0.013498316953033454), None),
            ((122, 40, 66, -0.016896390550754353), Some(BlockState::AIR)),
            ((122, 40, 68, -0.01994683889106233), Some(BlockState::AIR)),
            ((122, 40, 70, -0.022658183924480487), Some(BlockState::AIR)),
            ((122, 40, 72, -0.02460705550987633), Some(BlockState::AIR)),
            ((122, 40, 74, -0.02133677750482264), None),
            ((122, 60, 64, -0.02580014098083049), Some(*WATER_BLOCK)),
            ((122, 60, 66, -0.027410062228040422), Some(*WATER_BLOCK)),
            ((122, 60, 68, -0.02425659570836858), Some(*WATER_BLOCK)),
            ((122, 60, 70, -0.03261718168256943), Some(*WATER_BLOCK)),
            ((122, 60, 72, -0.04369665936638442), Some(*WATER_BLOCK)),
            ((122, 60, 74, -0.04490159197647781), Some(*WATER_BLOCK)),
            ((122, 80, 64, -0.37247525946547166), Some(BlockState::AIR)),
            ((122, 80, 66, -0.3727266378002749), Some(BlockState::AIR)),
            ((122, 80, 68, -0.36804742745663505), Some(BlockState::AIR)),
            ((122, 80, 70, -0.3736723706537362), Some(BlockState::AIR)),
            ((122, 80, 72, -0.3860951288334311), Some(BlockState::AIR)),
            ((122, 80, 74, -0.3923721309133264), Some(BlockState::AIR)),
            ((122, 100, 64, -0.4583333333333333), Some(BlockState::AIR)),
            ((122, 100, 66, -0.4583333333333333), Some(BlockState::AIR)),
            ((122, 100, 68, -0.4583333333333333), Some(BlockState::AIR)),
            ((122, 100, 70, -0.4583333333333333), Some(BlockState::AIR)),
            ((122, 100, 72, -0.4583333333333333), Some(BlockState::AIR)),
            ((122, 100, 74, -0.4583333333333333), Some(BlockState::AIR)),
            ((124, -100, 64, 0.037482421875), None),
            ((124, -100, 66, 0.037482421875), None),
            ((124, -100, 68, 0.037482421875), None),
            ((124, -100, 70, 0.037482421875), None),
            ((124, -100, 72, 0.037482421875), None),
            ((124, -100, 74, 0.037482421875), None),
            ((124, -80, 64, 0.037482421875), None),
            ((124, -80, 66, 0.037482421875), None),
            ((124, -80, 68, 0.037482421875), None),
            ((124, -80, 70, 0.037482421875), None),
            ((124, -80, 72, 0.037482421875), None),
            ((124, -80, 74, 0.037482421875), None),
            ((124, -60, 64, 0.05027131800090962), None),
            ((124, -60, 66, 0.0509584844305151), None),
            ((124, -60, 68, 0.05166719216129481), None),
            ((124, -60, 70, 0.05239749814810798), None),
            ((124, -60, 72, 0.053150241639753654), None),
            ((124, -60, 74, 0.053923063702221205), None),
            ((124, -40, 64, 0.1288961855391929), None),
            ((124, -40, 66, 0.13348604127036617), None),
            ((124, -40, 68, 0.13753360917037172), None),
            ((124, -40, 70, 0.1415440368478666), None),
            ((124, -40, 72, 0.1455395599824845), None),
            ((124, -40, 74, 0.1495006614184435), None),
            ((124, -20, 64, 0.12612367959767024), None),
            ((124, -20, 66, 0.12830319402402945), None),
            ((124, -20, 68, 0.1304960062341348), None),
            ((124, -20, 70, 0.1327707947453995), None),
            ((124, -20, 72, 0.13519345838131658), None),
            ((124, -20, 74, 0.13781110986582973), None),
            ((124, 0, 64, -0.009009809178163559), Some(*WATER_BLOCK)),
            ((124, 0, 66, -0.007660237459532607), Some(*WATER_BLOCK)),
            ((124, 0, 68, -0.0061446463166489424), Some(*WATER_BLOCK)),
            ((124, 0, 70, -0.004442459854201204), Some(*WATER_BLOCK)),
            ((124, 0, 72, -0.0025318494505197223), Some(*WATER_BLOCK)),
            ((124, 0, 74, -4.216225767843497E-4), None),
            ((124, 20, 64, 0.06277697412858188), None),
            ((124, 20, 66, 0.06182400117210263), None),
            ((124, 20, 68, 0.060920470163534336), None),
            ((124, 20, 70, 0.06008166953195572), None),
            ((124, 20, 72, 0.05931163578048011), None),
            ((124, 20, 74, 0.05862261977236657), None),
            ((124, 40, 64, -0.004596793013348681), None),
            ((124, 40, 66, -0.007881293688553023), None),
            ((124, 40, 68, -0.010851313478373932), None),
            ((124, 40, 70, -0.013513605008223933), Some(*WATER_BLOCK)),
            ((124, 40, 72, -0.015880866729427373), Some(*WATER_BLOCK)),
            ((124, 40, 74, -0.017978317799117856), Some(*WATER_BLOCK)),
            ((124, 60, 64, -0.033729060298063454), Some(*WATER_BLOCK)),
            ((124, 60, 66, -0.04062740064249005), Some(*WATER_BLOCK)),
            ((124, 60, 68, -0.03660634922712756), Some(*WATER_BLOCK)),
            ((124, 60, 70, -0.04106936165998065), Some(*WATER_BLOCK)),
            ((124, 60, 72, -0.048715160337165046), Some(*WATER_BLOCK)),
            ((124, 60, 74, -0.053817378732386144), Some(*WATER_BLOCK)),
            ((124, 80, 64, -0.378513110274629), Some(BlockState::AIR)),
            ((124, 80, 66, -0.37887533037235366), Some(BlockState::AIR)),
            ((124, 80, 68, -0.3755672366866089), Some(BlockState::AIR)),
            ((124, 80, 70, -0.3806264904596738), Some(BlockState::AIR)),
            ((124, 80, 72, -0.39139114552312176), Some(BlockState::AIR)),
            ((124, 80, 74, -0.39905004304932734), Some(BlockState::AIR)),
            ((124, 100, 64, -0.4583333333333333), Some(BlockState::AIR)),
            ((124, 100, 66, -0.4583333333333333), Some(BlockState::AIR)),
            ((124, 100, 68, -0.4583333333333333), Some(BlockState::AIR)),
            ((124, 100, 70, -0.4583333333333333), Some(BlockState::AIR)),
            ((124, 100, 72, -0.4583333333333333), Some(BlockState::AIR)),
            ((124, 100, 74, -0.4583333333333333), Some(BlockState::AIR)),
            ((126, -100, 64, 0.037482421875), None),
            ((126, -100, 66, 0.037482421875), None),
            ((126, -100, 68, 0.037482421875), None),
            ((126, -100, 70, 0.037482421875), None),
            ((126, -100, 72, 0.037482421875), None),
            ((126, -100, 74, 0.037482421875), None),
            ((126, -80, 64, 0.037482421875), None),
            ((126, -80, 66, 0.037482421875), None),
            ((126, -80, 68, 0.037482421875), None),
            ((126, -80, 70, 0.037482421875), None),
            ((126, -80, 72, 0.037482421875), None),
            ((126, -80, 74, 0.037482421875), None),
            ((126, -60, 64, 0.05024330978368198), None),
            ((126, -60, 66, 0.05093542123713047), None),
            ((126, -60, 68, 0.05164825745773993), None),
            ((126, -60, 70, 0.052380776473173525), None),
            ((126, -60, 72, 0.053133618951577574), None),
            ((126, -60, 74, 0.05390538149459787), None),
            ((126, -40, 64, 0.1308942633558083), None),
            ((126, -40, 66, 0.1355722846035891), None),
            ((126, -40, 68, 0.13977234300629068), None),
            ((126, -40, 70, 0.14391148475400958), None),
            ((126, -40, 72, 0.1480252139578508), None),
            ((126, -40, 74, 0.15208907413316627), None),
            ((126, -20, 64, 0.13001056656809973), None),
            ((126, -20, 66, 0.13213006749070327), None),
            ((126, -20, 68, 0.1342494739710899), None),
            ((126, -20, 70, 0.13641807828631622), None),
            ((126, -20, 72, 0.13869641277763253), None),
            ((126, -20, 74, 0.1411390651002113), None),
            ((126, 0, 64, -0.010355206926252296), Some(*WATER_BLOCK)),
            ((126, 0, 66, -0.009043874021560911), Some(*WATER_BLOCK)),
            ((126, 0, 68, -0.007576245457331987), Some(*WATER_BLOCK)),
            ((126, 0, 70, -0.005915269354878528), Some(*WATER_BLOCK)),
            ((126, 0, 72, -0.0040306175772153365), Some(*WATER_BLOCK)),
            ((126, 0, 74, -0.0019328609472880898), Some(*WATER_BLOCK)),
            ((126, 20, 64, 0.0700499260773044), None),
            ((126, 20, 66, 0.06908003164862005), None),
            ((126, 20, 68, 0.0681425614589177), None),
            ((126, 20, 70, 0.06723915989832648), None),
            ((126, 20, 72, 0.066359174495176), None),
            ((126, 20, 74, 0.06551116407146489), None),
            ((126, 40, 64, 0.004497597038868666), None),
            ((126, 40, 66, 0.0013502912778844962), None),
            ((126, 40, 68, -0.0015191728313191184), Some(*WATER_BLOCK)),
            ((126, 40, 70, -0.0041188588354404134), Some(*WATER_BLOCK)),
            ((126, 40, 72, -0.006463772144671846), Some(*WATER_BLOCK)),
            ((126, 40, 74, -0.008581034519921562), Some(*WATER_BLOCK)),
            ((126, 60, 64, -0.03471424652823008), Some(*WATER_BLOCK)),
            ((126, 60, 66, -0.04732045558891548), Some(*WATER_BLOCK)),
            ((126, 60, 68, -0.04568337003176991), Some(*WATER_BLOCK)),
            ((126, 60, 70, -0.0428377824231183), Some(*WATER_BLOCK)),
            ((126, 60, 72, -0.04738820166968918), Some(*WATER_BLOCK)),
            ((126, 60, 74, -0.05663750895047857), Some(*WATER_BLOCK)),
            ((126, 80, 64, -0.37931742687180287), Some(BlockState::AIR)),
            ((126, 80, 66, -0.38265481838544235), Some(BlockState::AIR)),
            ((126, 80, 68, -0.3808041835281554), Some(BlockState::AIR)),
            ((126, 80, 70, -0.38160238129796925), Some(BlockState::AIR)),
            ((126, 80, 72, -0.387746448733821), Some(BlockState::AIR)),
            ((126, 80, 74, -0.3990668807989283), Some(BlockState::AIR)),
            ((126, 100, 64, -0.4583333333333333), Some(BlockState::AIR)),
            ((126, 100, 66, -0.4583333333333333), Some(BlockState::AIR)),
            ((126, 100, 68, -0.4583333333333333), Some(BlockState::AIR)),
            ((126, 100, 70, -0.4583333333333333), Some(BlockState::AIR)),
            ((126, 100, 72, -0.4583333333333333), Some(BlockState::AIR)),
            ((126, 100, 74, -0.4583333333333333), Some(BlockState::AIR)),
        ];

        for ((x, y, z, sample), result) in values {
            let pos = &NoisePos::Unblended(UnblendedNoisePos::new(x, y, z));
            assert_eq!(
                aquifer.apply_internal(pos, &env, &mut funcs, sample),
                result
            );
        }
    }
}
