use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use data::BlendingData;
use enum_dispatch::enum_dispatch;
use pumpkin_core::{
    math::{hypot, magnitude},
    random::{xoroshiro128::Xoroshiro, RandomGenerator, RandomImpl},
};

use crate::{biome::Biome, height::HeightLimitViewImpl};

use super::{
    biome_coords,
    chunk::{Chunk, ChunkPos},
    noise::{
        density::NoisePosImpl,
        lerp,
        perlin::{DoublePerlinNoiseParameters, DoublePerlinNoiseSampler},
    },
    supplier::{BiomeSupplier, StaticBiomeSupplier},
};

pub mod data;

static OFFSET_NOISE: LazyLock<DoublePerlinNoiseSampler> = LazyLock::new(|| {
    DoublePerlinNoiseSampler::new(
        &mut RandomGenerator::Xoroshiro(Xoroshiro::from_seed(42)),
        &DoublePerlinNoiseParameters::new(-3, &[1f64; 4]),
    )
});
const BLENDING_BIOME_DISTANCE_THRESHOLD: i32 = (7 << 2) - 1;
const BLENDING_CHUNK_DISTANCE_THRESHOLD: i32 = (BLENDING_BIOME_DISTANCE_THRESHOLD + 3) >> 2;
const CLOSE_BLENDING_DISTANCE_THRESHOLD: i32 = 5 >> 2;

#[derive(Clone)]
pub struct BlendResult {
    alpha: f64,
    offset: f64,
}

impl BlendResult {
    pub fn new(alpha: f64, offset: f64) -> Self {
        Self { alpha, offset }
    }

    pub fn alpha(&self) -> f64 {
        self.alpha
    }

    pub fn offset(&self) -> f64 {
        self.offset
    }
}

#[enum_dispatch(BlenderImpl)]
#[derive(Clone)]
pub enum Blender {
    None(NoBlendBlender),
    Standard(StandardBlender),
}

#[enum_dispatch]
pub trait BlenderImpl {
    fn apply_blend_density(&self, pos: &impl NoisePosImpl, density: f64) -> f64;
    fn calculate(&self, block_x: i32, block_z: i32) -> BlendResult;
    fn biome_supplier(&self, supplier: &BiomeSupplier) -> BiomeSupplier;
}

#[derive(Clone)]
pub struct NoBlendBlender {}
impl BlenderImpl for NoBlendBlender {
    fn calculate(&self, _block_x: i32, _block_z: i32) -> BlendResult {
        BlendResult {
            alpha: 1f64,
            offset: 0f64,
        }
    }

    fn apply_blend_density(&self, _pos: &impl NoisePosImpl, density: f64) -> f64 {
        density
    }

    fn biome_supplier(&self, supplier: &BiomeSupplier) -> BiomeSupplier {
        supplier.clone()
    }
}

#[derive(Clone)]
struct StandardBlender {
    blend_data: Arc<HashMap<u64, BlendingData>>,
    close_blend_data: Arc<HashMap<u64, BlendingData>>,
}

#[derive(Clone)]
enum BlendingSampleType {
    Height,
    Density,
}

impl StandardBlender {
    fn new(
        blend_data: HashMap<u64, BlendingData>,
        close_blend_data: HashMap<u64, BlendingData>,
    ) -> Self {
        Self {
            blend_data: Arc::new(blend_data),
            close_blend_data: Arc::new(close_blend_data),
        }
    }

    fn blend_offset(height: f64) -> f64 {
        let e = height + 0.5f64;
        let f = ((e % 8f64) + 8f64) % 8f64;
        (32f64 * (e - 128f64) - 3f64 * (e - 128f64) * f + 3f64 * f * f)
            / (128f64 * (32f64 - 3f64 * f))
    }

    fn sample_closest(
        &self,
        sample_type: BlendingSampleType,
        biome_x: i32,
        biome_y: i32,
        biome_z: i32,
    ) -> f64 {
        let i = biome_coords::to_chunk(biome_x);
        let j = biome_coords::to_chunk(biome_z);

        let bl = (biome_x & 3) == 0;
        let bl2 = (biome_z & 3) == 0;

        let mut d = self.sample(sample_type.clone(), i, j, biome_x, biome_y, biome_z);

        if d == f64::MAX {
            if bl && bl2 {
                d = self.sample(sample_type.clone(), i - 1, j - 1, biome_x, biome_y, biome_z);
            }

            if d == f64::MAX && bl {
                d = self.sample(sample_type.clone(), i - 1, j, biome_x, biome_y, biome_z);
            }

            if d == f64::MAX && bl2 {
                d = self.sample(sample_type.clone(), i, j - 1, biome_x, biome_y, biome_z);
            }
        }

        d
    }

    fn sample(
        &self,
        sample_type: BlendingSampleType,
        chunk_x: i32,
        chunk_z: i32,
        biome_x: i32,
        biome_y: i32,
        biome_z: i32,
    ) -> f64 {
        if let Some(blending_data) = self
            .blend_data
            .get(&ChunkPos::new(chunk_x, chunk_z).to_long())
        {
            match sample_type {
                BlendingSampleType::Height => blending_data.height(
                    (biome_x - biome_coords::from_chunk(chunk_x)) as usize,
                    biome_y as usize,
                    (biome_z - biome_coords::from_chunk(chunk_z)) as usize,
                ),
                BlendingSampleType::Density => blending_data.collidable_block_density(
                    (biome_x - biome_coords::from_chunk(chunk_x)) as usize,
                    biome_y,
                    (biome_z - biome_coords::from_chunk(chunk_z)) as usize,
                ),
            }
        } else {
            f64::MAX
        }
    }

    fn blend_biome(&self, x: i32, y: i32, z: i32) -> Option<Biome> {
        for (k, v) in self.blend_data.iter() {
            let biome_x = biome_coords::from_chunk(ChunkPos::packed_x(*k));
            let biome_z = biome_coords::from_chunk(ChunkPos::packed_z(*k));

            if y >= biome_coords::from_block(v.height_limit.bottom_y())
                && y < biome_coords::from_block(v.height_limit.top_y())
            {
                let mut val = f64::INFINITY;
                let mut biome: Option<Biome> = None;

                let i = biome_coords::from_block(v.height_limit.bottom_y());

                for (j, biome_list) in v.biomes.iter().enumerate() {
                    if let Some(biome_list) = biome_list {
                        if let Some(internal_biome) = biome_list.get(i as usize) {
                            let biome_x = biome_x + BlendingData::x(j) as i32;
                            let biome_z = biome_z + BlendingData::z(j) as i32;

                            let dx = hypot((x - biome_x) as f64, (z - biome_z) as f64);
                            if dx <= BLENDING_BIOME_DISTANCE_THRESHOLD as f64 && dx < val {
                                val = dx;
                                biome = Some(*internal_biome)
                            }
                        }
                    }
                }

                if val < f64::MAX {
                    let d = OFFSET_NOISE.sample(x as f64, 0f64, z as f64) * 12f64;
                    let e = ((val + d) / (BLENDING_BIOME_DISTANCE_THRESHOLD + 1) as f64)
                        .clamp(0f64, 1f64);
                    if e <= 0.5f64 {
                        return Some(biome.unwrap());
                    }
                }
            }
        }

        None
    }
}

impl BlenderImpl for StandardBlender {
    fn calculate(&self, block_x: i32, block_z: i32) -> BlendResult {
        let i = biome_coords::from_block(block_x);
        let j = biome_coords::from_block(block_z);

        let d = self.sample_closest(BlendingSampleType::Height, i, 0, j);
        if d != f64::MAX {
            BlendResult {
                alpha: 0f64,
                offset: Self::blend_offset(d),
            }
        } else {
            let mut val1 = 0f64;
            let mut val2 = 0f64;
            let mut val3 = f64::INFINITY;

            for (chunk_pos, data) in self.blend_data.iter() {
                let biome_x = biome_coords::from_chunk(ChunkPos::packed_x(*chunk_pos));
                let biome_z = biome_coords::from_chunk(ChunkPos::packed_z(*chunk_pos));

                for (index, height) in data.surface_heights.iter().enumerate() {
                    if *height != f64::MAX {
                        let biome_x = biome_x + BlendingData::x(index) as i32;
                        let biome_z = biome_z + BlendingData::z(index) as i32;

                        let dx = hypot((i - biome_x) as f64, (j - biome_z) as f64);
                        if dx <= BLENDING_BIOME_DISTANCE_THRESHOLD as f64 {
                            if dx < val3 {
                                val3 = dx;
                            }
                            let ex = 1f64 / (dx * dx * dx * dx);
                            val2 += *height * ex;
                            val1 += ex;
                        }
                    }
                }
            }

            if val3 == f64::INFINITY {
                BlendResult {
                    alpha: 1f64,
                    offset: 0f64,
                }
            } else {
                let e = val2 / val1;
                let f = (val3 / (BLENDING_BIOME_DISTANCE_THRESHOLD + 1) as f64).clamp(0f64, 1f64);
                let f = 3f64 * f * f - 2f64 * f * f * f;
                BlendResult {
                    alpha: f,
                    offset: Self::blend_offset(e),
                }
            }
        }
    }

    fn apply_blend_density(&self, pos: &impl NoisePosImpl, density: f64) -> f64 {
        let i = biome_coords::from_block(pos.x());
        let j = pos.y() / 8;
        let k = biome_coords::from_block(pos.z());

        let d = self.sample_closest(BlendingSampleType::Density, i, j, k);

        if d != f64::MAX {
            d
        } else {
            let mut val1 = 0f64;
            let mut val2 = 0f64;
            let mut val3 = f64::INFINITY;

            for (chunk_pos, data) in self.close_blend_data.iter() {
                let biome_x = biome_coords::from_chunk(ChunkPos::packed_z(*chunk_pos));
                let biome_y = biome_coords::from_chunk(ChunkPos::packed_z(*chunk_pos));
                let min_half_section_y = j - 1;
                let max_half_section_y = j + 1;

                let one_above = data.bottom_half_section_y() + 1;
                let j = 0.max(min_half_section_y - one_above);
                let k = data
                    .vertical_half_section_count()
                    .min(max_half_section_y - one_above);

                for (index, density_vec) in data.collidable_block_densities.iter().enumerate() {
                    if let Some(density_vec) = density_vec {
                        let m = biome_x + BlendingData::x(index) as i32;
                        let n = biome_y + BlendingData::z(index) as i32;

                        for o in j..k {
                            let biome_x = m;
                            let half_section_y = o + one_above;
                            let biome_z = n;
                            let density = density_vec[o as usize] * 0.1f64;

                            let dx = magnitude(
                                (i - biome_x) as f64,
                                ((j - half_section_y) * 2) as f64,
                                (k - biome_z) as f64,
                            );

                            if dx <= 2f64 {
                                if dx < val3 {
                                    val3 = dx;
                                }

                                let ex = 1f64 / (dx * dx * dx * dx);
                                val2 += density * ex;
                                val1 += ex;
                            }
                        }
                    }
                }
            }

            if val3 == f64::INFINITY {
                density
            } else {
                let e = val2 / val1;
                let f = (val3 / 3f64).clamp(0f64, 1f64);
                lerp(f, e, density)
            }
        }
    }

    fn biome_supplier(&self, _supplier: &BiomeSupplier) -> BiomeSupplier {
        BiomeSupplier::Static(StaticBiomeSupplier {})
    }
}
