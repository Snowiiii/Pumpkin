use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Cell, RefCell},
    collections::HashMap,
    default,
    ops::DerefMut,
    rc::Rc,
    sync::Arc,
};

use crate::world_gen::{
    biome_coords,
    blender::{BlendResult, Blender},
    chunk::GenerationShapeConfig,
    noise::lerp3,
    sampler,
};

use super::{
    density::{
        Applier, ApplierImpl, DensityFunction, DensityFunctionImpl, NoisePos, NoisePosImpl,
        UnblendedNoisePos, VisitorImpl, WrapperFunction, WrapperType,
    },
    lerp,
};

struct InterpolationApplier<'a> {
    parent_sampler: &'a mut NoisePos<'a>,
}

impl<'a> ApplierImpl<'a> for InterpolationApplier<'a> {
    fn at(&self, index: i32) -> NoisePos<'a> {
        /*
        match self.parent_sampler {
            NoisePos::ChunkNoise(sampler) => {
                sampler.start_block_y =
                    (index + sampler.minimum_cell_y) * sampler.vertical_cell_block_count;
                sampler.sample_unique += 1;
                sampler.cell_block_y = 0;
                sampler.index = index;

                NoisePos::ChunkNoise(sampler)
            }
            _ => unreachable!(),
        }
        */
        unimplemented!()
    }

    fn fill(&self, _densities: &[f64], function: &DensityFunction<'a>) -> Vec<f64> {
        // This is hella bad... had to get around borrow checker
        // TODO: Fix

        /*
            let cell_count = match self.parent_sampler {
                NoisePos::ChunkNoise(sampler) => sampler.vertical_cell_count,
                _ => unreachable!(),
            };

            let mut result: Vec<f64> = Vec::new();
            for i in 0..=cell_count {
                match self.parent_sampler {
                    NoisePos::ChunkNoise(sampler) => {
                        sampler.start_block_y =
                            (i + sampler.minimum_cell_y) * sampler.vertical_cell_block_count;
                        sampler.sample_unique += 1;
                        sampler.cell_block_y = 0;
                        sampler.index = i;
                        // collector.push(function.sample(self.parent_sampler));
                    }

                    _ => unreachable!(),
                }

                let sample = match self.parent_sampler {
                    NoisePos::ChunkNoise(_) => function.sample(self.parent_sampler),
                    _ => unreachable!(),
                };

                result.push(sample);
            }

            result
        */
        unimplemented!()
    }
}

#[derive(Clone)]
pub(crate) struct InterpolatorDensityFunction<'a> {
    parent_sampler: &'a ChunkNoiseSampler<'a>,
    start_buffer: Vec<Vec<f64>>,
    end_buffer: Vec<Vec<f64>>,
    delegate: Arc<DensityFunction<'a>>,
    x0y0z0: f64,
    x0y0z1: f64,
    x1y0z0: f64,
    x1y0z1: f64,
    x0y1z0: f64,
    x0y1z1: f64,
    x1y1z0: f64,
    x1y1z1: f64,
    x0z0: f64,
    x1z0: f64,
    x0z1: f64,
    x1z1: f64,
    z0: f64,
    z1: f64,
    result: f64,
}

impl<'a> InterpolatorDensityFunction<'a> {
    // TODO: All instances automatically added to Sampler
    fn new(
        parent_sampler: &'a ChunkNoiseSampler<'a>,
        delegate: Arc<DensityFunction<'a>>,
        vertical_cell_count: i32,
        horizontal_cell_count: i32,
    ) -> Self {
        Self {
            parent_sampler,
            start_buffer: Self::create_buffer(vertical_cell_count, horizontal_cell_count),
            end_buffer: Self::create_buffer(vertical_cell_count, horizontal_cell_count),
            delegate,
            x0y0z0: 0f64,
            x0y0z1: 0f64,
            x1y0z0: 0f64,
            x1y0z1: 0f64,
            x0y1z0: 0f64,
            x0y1z1: 0f64,
            x1y1z0: 0f64,
            x1y1z1: 0f64,
            x0z0: 0f64,
            x1z0: 0f64,
            x0z1: 0f64,
            x1z1: 0f64,
            z0: 0f64,
            z1: 0f64,
            result: 0f64,
        }
    }

    fn on_sampled_cell_corners(&mut self, cell_y: usize, cell_z: usize) {
        self.x0y0z0 = self.start_buffer[cell_z][cell_y];
        self.x0y0z1 = self.start_buffer[cell_z + 1][cell_y];
        self.x1y0z0 = self.end_buffer[cell_z][cell_y];
        self.x1y0z1 = self.end_buffer[cell_z + 1][cell_y];
        self.x0y1z0 = self.start_buffer[cell_z][cell_y + 1];
        self.x0y1z1 = self.start_buffer[cell_z + 1][cell_y + 1];
        self.x1y1z0 = self.end_buffer[cell_z][cell_y + 1];
        self.x1y1z1 = self.end_buffer[cell_z + 1][cell_y + 1];
    }

    fn interpolate_y(&mut self, delta: f64) {
        self.x0z0 = lerp(delta, self.x0y0z0, self.x0y1z0);
        self.x1z0 = lerp(delta, self.x1y0z0, self.x1y1z0);
        self.x0z1 = lerp(delta, self.x0y0z1, self.x0y1z1);
        self.x1z1 = lerp(delta, self.x1y0z1, self.x1y1z1);
    }

    fn interpolate_x(&mut self, delta: f64) {
        self.z0 = lerp(delta, self.x0z0, self.x1z0);
        self.z1 = lerp(delta, self.x0z1, self.x1z1);
    }

    fn interpolate_z(&mut self, delta: f64) {
        self.result = lerp(delta, self.z0, self.z1);
    }

    fn swap_buffers(&mut self) {
        let ds = self.start_buffer.clone();
        self.start_buffer = self.end_buffer.clone();
        self.start_buffer = ds;
    }

    fn create_buffer(size_z: i32, size_x: i32) -> Vec<Vec<f64>> {
        let i = size_x + 1;
        let j = size_z + 1;

        let mut result = Vec::<Vec<f64>>::new();
        for _ in 0..i {
            result.push((0..j).map(|_| 0f64).collect())
        }

        result
    }
}

impl<'a> DensityFunctionImpl<'a> for InterpolatorDensityFunction<'a> {
    fn sample(&self, pos: &super::density::NoisePos) -> f64 {
        if let NoisePos::ChunkNoise(sampler) = pos {
            if !std::ptr::eq(sampler, self.parent_sampler) {
                return self.delegate.sample(pos);
            }
        }

        assert!(self.parent_sampler.interpolating, "Not interpolating");
        if self.parent_sampler.sampling_for_caches {
            lerp3(
                self.parent_sampler.cell_block_x as f64
                    / self.parent_sampler.horizontal_cell_block_count as f64,
                self.parent_sampler.cell_block_y as f64
                    / self.parent_sampler.vertical_cell_block_count as f64,
                self.parent_sampler.cell_block_z as f64
                    / self.parent_sampler.horizontal_cell_block_count as f64,
                self.x0y0z0,
                self.x1y0z0,
                self.x0y1z0,
                self.x1y1z0,
                self.x0y0z1,
                self.x1y0z1,
                self.x0y1z1,
                self.x1y1z1,
            )
        } else {
            self.result
        }
    }

    fn apply(&'a self, visitor: &'a super::density::Visitor) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
            self.delegate.clone(),
            WrapperType::Interpolated,
        ))))
    }

    fn fill(&self, densities: &[f64], applier: &Applier) -> Vec<f64> {
        if self.parent_sampler.sampling_for_caches {
            applier.fill(densities, &DensityFunction::Interpolator(self.clone()))
        } else {
            self.delegate.fill(densities, applier)
        }
    }

    fn min(&self) -> f64 {
        self.delegate.min()
    }

    fn max(&self) -> f64 {
        self.delegate.max()
    }
}

#[derive(Clone)]
pub(crate) struct FlatCacheDensityFunction<'a> {
    parent_sampler: &'a ChunkNoiseSampler<'a>,
    delegate: Arc<DensityFunction<'a>>,
    cache: Vec<Vec<f64>>,
}

impl<'a> FlatCacheDensityFunction<'a> {
    fn new(
        parent_sampler: &'a ChunkNoiseSampler<'a>,
        delegate: Arc<DensityFunction<'a>>,
        biome_start_x: i32,
        biome_start_z: i32,
        horizontal_biome_end: usize,
        sample: bool,
    ) -> Self {
        let mut cache: Vec<Vec<f64>> = Vec::new();
        for _ in 0..(horizontal_biome_end + 1) {
            cache.push((0..(horizontal_biome_end + 1)).map(|_| 0f64).collect())
        }

        if sample {
            for i in 0..=horizontal_biome_end {
                let j = biome_start_x + i as i32;
                let k = biome_coords::to_block(j);

                for l in 0..=horizontal_biome_end {
                    let m = biome_start_z + l as i32;
                    let n = biome_coords::to_block(m);
                    cache[i][l] =
                        delegate.sample(&NoisePos::Unblended(UnblendedNoisePos::new(k, 0, n)));
                }
            }
        }

        Self {
            parent_sampler,
            delegate,
            cache,
        }
    }
}

impl<'a> DensityFunctionImpl<'a> for FlatCacheDensityFunction<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        let i = biome_coords::from_block(pos.x());
        let j = biome_coords::from_block(pos.z());
        let k = i - self.parent_sampler.start_biome_x;
        let l = j - self.parent_sampler.start_biome_z;
        if k >= 0 && l >= 0 && (k as usize) < self.cache.len() && (l as usize) < self.cache.len() {
            self.cache[k as usize][l as usize]
        } else {
            self.delegate.sample(pos)
        }
    }

    fn fill(&self, densities: &[f64], applier: &Applier) -> Vec<f64> {
        applier.fill(densities, &DensityFunction::FlatCache(self.clone()))
    }

    fn apply(&'a self, visitor: &'a super::density::Visitor) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
            self.delegate.apply(visitor),
            WrapperType::CacheFlat,
        ))))
    }

    fn min(&self) -> f64 {
        self.delegate.min()
    }

    fn max(&self) -> f64 {
        self.delegate.max()
    }
}

struct CellCacheDensityFunction {}

enum BlockStateSampler {}

pub struct ChunkNoiseSampler<'a> {
    shape_config: GenerationShapeConfig,
    horizontal_cell_count: i32,
    vertical_cell_count: i32,
    minimum_cell_y: i32,
    start_cell_x: i32,
    start_cell_z: i32,
    start_biome_x: i32,
    start_biome_z: i32,
    interpolators: Vec<InterpolatorDensityFunction<'a>>,
    caches: Vec<CellCacheDensityFunction>,
    surface_height_estimate_cache: HashMap<u64, i32>,
    initial_density: Arc<DensityFunction<'a>>,
    block_state_sampler: BlockStateSampler,
    blender: Blender,
    cached_blend_alpha_density_function: FlatCacheDensityFunction<'a>,
    cached_blend_offset_density_function: FlatCacheDensityFunction<'a>,
    last_blended_column_pos: u64,
    last_blending_result: BlendResult,
    horizontal_biome_end: i32,
    horizontal_cell_block_count: i32,
    vertical_cell_block_count: i32,
    interpolating: bool,
    sampling_for_caches: bool,
    start_block_x: i32,
    start_block_y: i32,
    start_block_z: i32,
    cell_block_x: i32,
    cell_block_y: i32,
    cell_block_z: i32,
    sample_unique: u64,
    cache_unique: u64,
    index: i32,
}

impl<'a> ApplierImpl<'a> for ChunkNoiseSampler<'a> {
    fn at(&self, index: i32) -> NoisePos {
        /*
        let j = index % self.horizontal_cell_block_count;
        let k = index % self.horizontal_cell_block_count;
        let l = index % self.horizontal_cell_block_count;
        let m = self.vertical_cell_block_count - 1 - (k / self.horizontal_cell_block_count);

        self.cell_block_x = l;
        self.cell_block_y = m;
        self.cell_block_z = j;
        self.index = index;
        */
        unimplemented!()
    }

    fn fill(&self, densities: &[f64], function: &DensityFunction<'a>) -> Vec<f64> {
        unimplemented!()
    }
}

impl<'a> NoisePosImpl for ChunkNoiseSampler<'a> {
    fn z(&self) -> i32 {
        unimplemented!()
    }

    fn y(&self) -> i32 {
        unimplemented!()
    }

    fn x(&self) -> i32 {
        unimplemented!()
    }
}
