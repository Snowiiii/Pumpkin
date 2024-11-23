use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
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

pub enum BlockStateSampler {
    Aquifer(AquiferSampler),
    Ore(OreVeinSampler),
    Chained(ChainedBlockStateSampler),
}

impl BlockStateSampler {
    pub fn sample(
        &mut self,
        pos: &NoisePos,
        state: &ChunkNoiseState,
        height_estimator: &mut ChunkNoiseDensityFunctions,
    ) -> Option<BlockState> {
        match self {
            Self::Aquifer(aquifer) => aquifer.apply(pos, state, height_estimator),
            Self::Ore(ore) => ore.sample(pos, state),
            Self::Chained(chained) => chained.sample(pos, state, height_estimator),
        }
    }
}

pub struct ChainedBlockStateSampler {
    samplers: Vec<BlockStateSampler>,
}

impl ChainedBlockStateSampler {
    pub fn new(samplers: Vec<BlockStateSampler>) -> Self {
        Self { samplers }
    }

    fn sample(
        &mut self,
        pos: &NoisePos,
        state: &ChunkNoiseState,
        height_estimator: &mut ChunkNoiseDensityFunctions,
    ) -> Option<BlockState> {
        self.samplers
            .iter_mut()
            .map(|sampler| sampler.sample(pos, state, height_estimator))
            .find(|state| state.is_some())
            .unwrap_or(None)
    }
}

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
    positions: Box<[Option<u64>]>,
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
            positions: vec![None; cache_size].into(),
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
        barrier_sample: &mut Option<f64>,
        level_1: FluidLevel,
        level_2: FluidLevel,
        env: &ChunkNoiseState,
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
                    if let Some(val) = barrier_sample {
                        *val
                    } else {
                        let sample = self.barrier_noise.sample_mut(pos, env);
                        *barrier_sample = Some(sample);
                        sample
                    }
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
        packed: u64,
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
            let x = block_x + section_coords::section_to_block(offset.x) as i32;
            let z = block_z + section_coords::section_to_block(offset.z) as i32;

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
        bl: bool,
        env: &ChunkNoiseState,
    ) -> i32 {
        let pos = NoisePos::Unblended(UnblendedNoisePos::new(block_x, block_y, block_z));

        let is_deep_dark = self.erosion.sample_mut(&pos, env) < -0.225f32 as f64
            && self.depth.sample_mut(&pos, env) > 0.9f32 as f64;

        let (d, e) = if is_deep_dark {
            (-1f64, -1f64)
        } else {
            let top_y = surface_height_estimate + 8 - block_y;
            let f = if bl {
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
}

impl AquiferSamplerImpl for WorldAquiferSampler {
    fn apply(
        &mut self,
        pos: &NoisePos,
        state: &ChunkNoiseState,
        height_estimator: &mut ChunkNoiseDensityFunctions,
    ) -> Option<BlockState> {
        let density = self.function.sample_mut(pos, state);
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
                let l = floor_div(i - 5, 16);
                let m = floor_div(j + 1, 12);
                let n = floor_div(k - 5, 16);

                let mut o = i32::MAX;
                let mut p = i32::MAX;
                let mut q = i32::MAX;
                let mut r = i32::MAX;

                let mut s = 0;
                let mut t = 0;
                let mut u = 0;
                let mut v = 0;

                for w in 0..=1 {
                    for x in -1..=1 {
                        for y in 0..=1 {
                            let z = l + w;
                            let aa = m + x;
                            let ab = n + y;

                            let ac = self.index(z, aa, ab);
                            let ad = self.positions[ac];

                            let ae = if let Some(packed) = ad {
                                packed
                            } else {
                                let mut random = self.random_deriver.split_pos(z, aa, ab);
                                let block_x = z * 16 + random.next_bounded_i32(10);
                                let block_y = aa * 12 + random.next_bounded_i32(9);
                                let block_z = ab * 16 + random.next_bounded_i32(10);

                                let packed =
                                    block_pos::packed(&Vector3::new(block_x, block_y, block_z));
                                self.positions[ac] = Some(packed);
                                packed
                            };

                            let af = block_pos::unpack_x(ae) - i;
                            let ag = block_pos::unpack_y(ae) - j;
                            let ah = block_pos::unpack_z(ae) - k;

                            let ai = af * af + ag * ag + ah * ah;
                            if o >= ai {
                                v = u;
                                u = t;
                                t = s;
                                s = ae;
                                r = q;
                                q = p;
                                p = o;
                                o = ai;
                            } else if p >= ai {
                                v = u;
                                u = t;
                                t = ae;
                                r = q;
                                q = p;
                                p = ai;
                            } else if q >= ai {
                                v = u;
                                u = ae;
                                r = q;
                                q = ai;
                            } else if r >= ai {
                                v = ae;
                                r = ai;
                            }
                        }
                    }
                }

                let fluid_level2 = self.get_water_level(s, height_estimator, state);
                let d = Self::max_distance(o, p);
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
                    let mut mutable_f64 = None;
                    let fluid_level3 = self.get_water_level(t, height_estimator, state);
                    let e = d * self.calculate_density(
                        pos,
                        &mut mutable_f64,
                        fluid_level2.clone(),
                        fluid_level3.clone(),
                        state,
                    );
                    if density + e > 0f64 {
                        None
                    } else {
                        let fluid_level4 = self.get_water_level(u, height_estimator, state);
                        let f = Self::max_distance(o, q);
                        if f > 0f64 {
                            let g = d
                                * f
                                * self.calculate_density(
                                    pos,
                                    &mut mutable_f64,
                                    fluid_level2,
                                    fluid_level4.clone(),
                                    state,
                                );
                            if density + g > 0f64 {
                                return None;
                            }
                        }

                        let g = Self::max_distance(p, q);
                        if g > 0f64 {
                            let h = d
                                * g
                                * self.calculate_density(
                                    pos,
                                    &mut mutable_f64,
                                    fluid_level3,
                                    fluid_level4,
                                    state,
                                );
                            if density + h > 0f64 {
                                return None;
                            }
                        }

                        //TODO Handle fluid tick
                        let _ = v;

                        Some(block_state)
                    }
                }
            }
        }
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
trait AquiferSamplerImpl {
    fn apply(
        &mut self,
        pos: &NoisePos,
        state: &ChunkNoiseState,
        height_estimator: &mut ChunkNoiseDensityFunctions,
    ) -> Option<BlockState>;
}

pub struct OreVeinSampler {
    vein_toggle: Box<dyn ComponentReference<ChunkNoiseState>>,
    vein_ridged: Box<dyn ComponentReference<ChunkNoiseState>>,
    vein_gap: Box<dyn ComponentReference<ChunkNoiseState>>,
    random_deriver: RandomDeriver,
}

impl OreVeinSampler {
    pub fn new(
        vein_toggle: Box<dyn ComponentReference<ChunkNoiseState>>,
        vein_ridged: Box<dyn ComponentReference<ChunkNoiseState>>,
        vein_gap: Box<dyn ComponentReference<ChunkNoiseState>>,
        random_deriver: RandomDeriver,
    ) -> Self {
        Self {
            vein_toggle,
            vein_ridged,
            vein_gap,
            random_deriver,
        }
    }
}

impl OreVeinSampler {
    fn sample(&mut self, pos: &NoisePos, state: &ChunkNoiseState) -> Option<BlockState> {
        let vein_sample = self.vein_toggle.sample_mut(pos, state);
        let vein_type: &VeinType = if vein_sample > 0f64 {
            &vein_type::COPPER
        } else {
            &vein_type::IRON
        };

        let block_y = pos.y();
        let max_to_y = vein_type.max_y - block_y;
        let y_to_min = block_y - vein_type.min_y;
        if (max_to_y >= 0) && (y_to_min >= 0) {
            let closest_to_bound = max_to_y.min(y_to_min);
            let mapped_diff = clamped_map(closest_to_bound as f64, 0f64, 20f64, -0.2f64, 0f64);
            let abs_sample = vein_sample.abs();
            if abs_sample + mapped_diff >= 0.4f32 as f64 {
                let mut random = self.random_deriver.split_pos(pos.x(), block_y, pos.z());
                if random.next_f32() <= 0.7f32 && self.vein_ridged.sample_mut(pos, state) < 0f64 {
                    let clamped_sample = clamped_map(
                        abs_sample,
                        0.4f32 as f64,
                        0.6f32 as f64,
                        0.1f32 as f64,
                        0.3f32 as f64,
                    );

                    return if (random.next_f32() as f64) < clamped_sample
                        && self.vein_gap.sample_mut(pos, state) > (-0.3f32 as f64)
                    {
                        Some(if random.next_f32() < 0.02f32 {
                            vein_type.raw_ore
                        } else {
                            vein_type.ore
                        })
                    } else {
                        Some(vein_type.stone)
                    };
                }
            }
        }
        None
    }
}

pub struct VeinType {
    ore: BlockState,
    raw_ore: BlockState,
    stone: BlockState,
    min_y: i32,
    max_y: i32,
}

// One of the victims of removing compile time blocks
pub mod vein_type {
    use super::*;

    lazy_static! {
        pub static ref COPPER: VeinType = VeinType {
            ore: BlockState::new("minecraft:copper_ore").unwrap(),
            raw_ore: BlockState::new("minecraft:raw_copper_block").unwrap(),
            stone: BlockState::new("minecraft:granite").unwrap(),
            min_y: 0,
            max_y: 50,
        };
        pub static ref IRON: VeinType = VeinType {
            ore: BlockState::new("minecraft:deepslate_iron_ore").unwrap(),
            raw_ore: BlockState::new("minecraft:raw_iron_block").unwrap(),
            stone: BlockState::new("minecraft:tuff").unwrap(),
            min_y: -60,
            max_y: -8,
        };
        pub static ref MIN_Y: i32 = IRON.min_y;
        pub static ref MAX_Y: i32 = COPPER.max_y;
    }
}
