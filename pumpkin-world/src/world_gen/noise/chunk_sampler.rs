use core::panic;
use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Cell, RefCell},
    collections::HashMap,
    default,
    ops::DerefMut,
    rc::Rc,
    sync::{Arc, LazyLock},
};

use enum_dispatch::enum_dispatch;
use itertools::Itertools;
use parking_lot::Mutex;

use crate::{
    block::BlockId,
    world_gen::{
        biome_coords,
        blender::{BlendResult, Blender, BlenderImpl},
        chunk::{Chunk, ChunkPos, GenerationShapeConfig, CHUNK_MARKER},
        implementation::aquifer::{
            self, AquiferSamplerImpl, AquiferSeaLevel, AquifierSampler, FluidLevelSampler,
        },
        noise::lerp3,
        sampler,
    },
};

use super::{
    config::NoiseConfig,
    density::{
        blend::{BlendAlphaFunction, BlendOffsetFunction},
        Applier, ApplierImpl, BeardifyerFunction, DensityFunction, DensityFunctionImpl, NoisePos,
        NoisePosImpl, UnblendedNoisePos, Visitor, VisitorImpl, WrapperFunction, WrapperType,
    },
    lerp,
    router::NoiseRouter,
};

pub(crate) struct InterpolationApplier<'a> {
    parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
}

impl<'a> ApplierImpl<'a> for InterpolationApplier<'a> {
    fn at(&self, index: usize) -> NoisePos<'a> {
        let index = index as i32;
        let mut sampler = self.parent_sampler.lock();
        sampler.start_block_y =
            (index + sampler.minimum_cell_y) * sampler.vertical_cell_block_count;
        sampler.sample_unique += 1;
        sampler.cell_block_y = 0;
        sampler.index = index;

        NoisePos::ChunkNoise(ChunkNoiseSamplerWrapper {
            parent_sampler: self.parent_sampler.clone(),
        })
    }

    fn fill(&self, densities: &mut [f64], function: &DensityFunction<'a>) {
        let mut sampler = self.parent_sampler.lock();

        for i in 0..=sampler.vertical_cell_count {
            sampler.start_block_y =
                (i + sampler.minimum_cell_y) * sampler.vertical_cell_block_count;
            sampler.sample_unique += 1;
            sampler.cell_block_y = 0;
            sampler.index = i;

            drop(sampler);
            let sample = function.sample(&NoisePos::ChunkNoise(ChunkNoiseSamplerWrapper {
                parent_sampler: self.parent_sampler.clone(),
            }));
            sampler = self.parent_sampler.lock();
            densities[sampler.index as usize] = sample;
        }
    }
}

#[derive(Clone)]
pub(crate) struct BlendOffsetDensityFunction<'a> {
    parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
}

impl<'a> DensityFunctionImpl<'a> for BlendOffsetDensityFunction<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        let mut sampler = self.parent_sampler.lock();
        sampler.calculate_blend_result(pos.x(), pos.z()).offset()
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
        applier.fill(
            densities,
            &DensityFunction::ChunkCacheBlendOffset(self.clone()),
        )
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        DensityFunction::BlendOffset(BlendOffsetFunction {}).apply(visitor)
    }

    fn max(&self) -> f64 {
        f64::INFINITY
    }

    fn min(&self) -> f64 {
        f64::NEG_INFINITY
    }
}

#[derive(Clone)]
pub(crate) struct BlendAlphaDensityFunction<'a> {
    parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
}

impl<'a> DensityFunctionImpl<'a> for BlendAlphaDensityFunction<'a> {
    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        DensityFunction::BlendAlpha(BlendAlphaFunction {}).apply(visitor)
    }

    fn sample(&self, pos: &NoisePos) -> f64 {
        let mut sampler = self.parent_sampler.lock();
        sampler.calculate_blend_result(pos.x(), pos.z()).alpha()
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
        applier.fill(
            densities,
            &DensityFunction::ChunkCacheBlendAlpha(self.clone()),
        )
    }

    fn min(&self) -> f64 {
        0f64
    }

    fn max(&self) -> f64 {
        1f64
    }
}

#[derive(Clone)]
pub(crate) struct InterpolatorDensityFunctionWrapper<'a> {
    wrapped: Arc<Mutex<InterpolatorDensityFunction<'a>>>,
}

impl<'a> DensityFunctionImpl<'a> for InterpolatorDensityFunctionWrapper<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        self.wrapped.lock().sample(pos)
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
        self.wrapped.lock().fill(densities, applier)
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        self.wrapped.lock().apply(visitor)
    }

    fn min(&self) -> f64 {
        self.wrapped.lock().min()
    }

    fn max(&self) -> f64 {
        self.wrapped.lock().max()
    }
}

#[derive(Clone)]
struct InterpolatorDensityFunction<'a> {
    parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
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
    fn new(
        parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
        delegate: Arc<DensityFunction<'a>>,
        vertical_cell_count: i32,
        horizontal_cell_count: i32,
    ) -> InterpolatorDensityFunctionWrapper<'a> {
        let result = Arc::new(Mutex::new(Self {
            parent_sampler: parent_sampler.clone(),
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
        }));
        let wrapper = InterpolatorDensityFunctionWrapper { wrapped: result };

        let mut sampler = parent_sampler.lock();
        sampler.interpolators.push(wrapper.clone());
        wrapper
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
            let other_lock = { sampler.parent_sampler.lock().uuid };

            let my_lock = { self.parent_sampler.lock().uuid };

            if other_lock == my_lock {
                return self.delegate.sample(pos);
            }
        }

        let sampler = self.parent_sampler.lock();
        assert!(sampler.interpolating, "Not interpolating");
        if sampler.sampling_for_caches {
            lerp3(
                sampler.cell_block_x as f64 / sampler.horizontal_cell_block_count as f64,
                sampler.cell_block_y as f64 / sampler.vertical_cell_block_count as f64,
                sampler.cell_block_z as f64 / sampler.horizontal_cell_block_count as f64,
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

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
            self.delegate.clone(),
            WrapperType::Interpolated,
        ))))
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
        let sampling = { self.parent_sampler.lock().sampling_for_caches };
        if sampling {
            applier.fill(
                densities,
                &DensityFunction::ChunkCacheInterpolator(InterpolatorDensityFunctionWrapper {
                    wrapped: Arc::new(Mutex::new(self.clone())),
                }),
            )
        } else {
            self.delegate.fill(densities, applier);
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
    parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
    delegate: Arc<DensityFunction<'a>>,
    cache: Vec<Vec<f64>>,
}

impl<'a> FlatCacheDensityFunction<'a> {
    fn new(
        parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
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
        let (start_biome_x, start_biome_z) = {
            let sampler = self.parent_sampler.lock();
            (sampler.start_biome_x, sampler.start_biome_z)
        };
        let k = i - start_biome_x;
        let l = j - start_biome_z;
        if k >= 0 && l >= 0 && (k as usize) < self.cache.len() && (l as usize) < self.cache.len() {
            self.cache[k as usize][l as usize]
        } else {
            self.delegate.sample(pos)
        }
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
        applier.fill(
            densities,
            &DensityFunction::ChunkCacheFlatCache(self.clone()),
        );
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
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

#[derive(Clone)]
pub(crate) struct CellCacheDensityFunctionWrapper<'a> {
    wrapped: Arc<CellCacheDensityFunction<'a>>,
}

impl<'a> DensityFunctionImpl<'a> for CellCacheDensityFunctionWrapper<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        self.wrapped.sample(pos)
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
        applier.fill(
            densities,
            &DensityFunction::ChunkCacheCellCache(self.clone()),
        );
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        self.wrapped.apply(visitor)
    }

    fn min(&self) -> f64 {
        self.wrapped.min()
    }

    fn max(&self) -> f64 {
        self.wrapped.max()
    }
}

#[derive(Clone)]
struct CellCacheDensityFunction<'a> {
    parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
    delegate: Arc<DensityFunction<'a>>,
    cache: Vec<f64>,
}

impl<'a> CellCacheDensityFunction<'a> {
    fn new(
        parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
        delegate: Arc<DensityFunction<'a>>,
    ) -> CellCacheDensityFunctionWrapper<'a> {
        let cache_size = {
            let sampler = parent_sampler.lock();
            sampler.horizontal_cell_block_count
                * sampler.horizontal_cell_block_count
                * sampler.vertical_cell_block_count
        };
        let result = Arc::new(Self {
            parent_sampler: parent_sampler.clone(),
            delegate,
            cache: (0..cache_size).map(|_| 0f64).collect(),
        });

        let wrapped = CellCacheDensityFunctionWrapper { wrapped: result };
        let mut sampler = parent_sampler.lock();
        sampler.caches.push(wrapped.clone());

        wrapped
    }
}

impl<'a> DensityFunctionImpl<'a> for CellCacheDensityFunction<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        if let NoisePos::ChunkNoise(sampler) = pos {
            let other_lock = { sampler.parent_sampler.lock().uuid };

            let my_lock = { self.parent_sampler.lock().uuid };

            if other_lock == my_lock {
                return self.delegate.sample(pos);
            }
        }

        let sampler = self.parent_sampler.lock();
        assert!(sampler.interpolating);
        let i = sampler.cell_block_x;
        let j = sampler.cell_block_y;
        let k = sampler.cell_block_z;

        if i >= 0
            && k >= 0
            && i < sampler.horizontal_cell_block_count
            && j < sampler.vertical_cell_block_count
            && k < sampler.horizontal_cell_block_count
        {
            self.cache[(((sampler.vertical_cell_block_count - 1 - j)
                * sampler.horizontal_cell_block_count
                + i)
                * sampler.horizontal_cell_block_count
                + k) as usize]
        } else {
            self.delegate.sample(pos)
        }
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {}

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
            self.delegate.apply(visitor),
            WrapperType::CacheCell,
        ))))
    }

    fn min(&self) -> f64 {
        self.delegate.min()
    }

    fn max(&self) -> f64 {
        self.delegate.max()
    }
}

struct CacheOnceDensityFunctionData {
    sample_unique_index: u64,
    cache_once_unique_index: u64,
    last_sampling_result: f64,
    cache: Option<Vec<f64>>,
}

#[derive(Clone)]
pub(crate) struct CacheOnceDensityFunction<'a> {
    parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
    delegate: Arc<DensityFunction<'a>>,
    data: Arc<Mutex<CacheOnceDensityFunctionData>>,
}

impl<'a> CacheOnceDensityFunction<'a> {
    fn new(
        parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
        delegate: Arc<DensityFunction<'a>>,
    ) -> Self {
        Self {
            parent_sampler,
            delegate,
            data: Arc::new(Mutex::new(CacheOnceDensityFunctionData {
                sample_unique_index: 0,
                cache_once_unique_index: 0,
                last_sampling_result: 0f64,
                cache: None,
            })),
        }
    }
}

impl<'a> DensityFunctionImpl<'a> for CacheOnceDensityFunction<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        let parent_sampler = self.parent_sampler.lock();
        let mut data = self.data.lock();

        if let NoisePos::ChunkNoise(sampler) = pos {
            let other_id = { sampler.parent_sampler.lock().uuid };
            let this_id = { self.parent_sampler.lock().uuid };
            if other_id != this_id {
                return self.delegate.sample(pos);
            }
        }

        if let Some(cache) = &data.cache {
            if data.cache_once_unique_index == parent_sampler.cache_unique {
                return cache[parent_sampler.index as usize];
            }
        }

        if data.sample_unique_index == parent_sampler.sample_unique {
            data.last_sampling_result
        } else {
            data.sample_unique_index = parent_sampler.sample_unique;
            let d = self.delegate.sample(pos);
            data.last_sampling_result = d;
            d
        }
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
        let parent_sampler = self.parent_sampler.lock();
        let mut data = self.data.lock();
        let cache_once_unique_index = data.cache_once_unique_index;

        if let Some(cache) = &mut data.cache {
            if parent_sampler.cache_unique == cache_once_unique_index {
                densities
                    .iter_mut()
                    .zip_eq(cache)
                    .for_each(|(val, temp)| *val = *temp);
                return;
            }
        }

        self.delegate.fill(densities, applier);
        data.cache_once_unique_index = parent_sampler.cache_unique;
        if let Some(cache) = &mut data.cache {
            if cache.len() == densities.len() {
                cache
                    .iter_mut()
                    .zip_eq(densities)
                    .for_each(|(val, temp)| *val = *temp);
                return;
            }
        }
        data.cache = Some(Vec::from_iter(densities.iter().map(|x| *x)));
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
            self.delegate.apply(visitor),
            WrapperType::CacheOnce,
        ))))
    }

    fn max(&self) -> f64 {
        self.delegate.max()
    }

    fn min(&self) -> f64 {
        self.delegate.min()
    }
}

struct Cache2DDensityFunctionData {
    last_sampling_column_pos: u64,
    last_sampling_result: f64,
}

#[derive(Clone)]
pub(crate) struct Cache2DDensityFunction<'a> {
    delegate: Arc<DensityFunction<'a>>,
    data: Arc<Mutex<Cache2DDensityFunctionData>>,
}

impl<'a> Cache2DDensityFunction<'a> {
    fn new(delegate: Arc<DensityFunction<'a>>) -> Self {
        Self {
            delegate,
            data: Arc::new(Mutex::new(Cache2DDensityFunctionData {
                last_sampling_column_pos: CHUNK_MARKER,
                last_sampling_result: 0f64,
            })),
        }
    }
}

impl<'a> DensityFunctionImpl<'a> for Cache2DDensityFunction<'a> {
    fn sample(&self, pos: &NoisePos) -> f64 {
        let i = pos.x();
        let j = pos.z();
        let l = ChunkPos::new(i, j).to_long();
        let mut data = self.data.lock();
        if data.last_sampling_column_pos == l {
            data.last_sampling_result
        } else {
            data.last_sampling_column_pos = l;
            drop(data);
            let d = self.delegate.sample(pos);
            let mut data = self.data.lock();
            data.last_sampling_result = d;
            d
        }
    }

    fn fill(&self, densities: &mut [f64], applier: &Applier<'a>) {
        self.delegate.fill(densities, applier)
    }

    fn apply(&self, visitor: &Visitor<'a>) -> Arc<DensityFunction<'a>> {
        visitor.apply(Arc::new(DensityFunction::Wrapper(WrapperFunction::new(
            self.delegate.clone(),
            WrapperType::Cache2D,
        ))))
    }

    fn min(&self) -> f64 {
        self.delegate.min()
    }

    fn max(&self) -> f64 {
        self.delegate.max()
    }
}

#[enum_dispatch(BlockStateSamplerImpl)]
enum BlockStateSampler<'a> {
    Aquifer(AquiferBlockStateSampler<'a>),
}

struct AquiferBlockStateSampler<'a> {
    aquifier: Arc<AquifierSampler>,
    function: Arc<DensityFunction<'a>>,
}

impl<'a> BlockStateSamplerImpl for AquiferBlockStateSampler<'a> {
    fn sample(&self, pos: &NoisePos) -> Option<BlockId> {
        self.aquifier.apply(pos, self.function.sample(pos))
    }
}

#[enum_dispatch]
trait BlockStateSamplerImpl {
    fn sample(&self, pos: &NoisePos) -> Option<BlockId>;
}

static SAMPLER_ID_COUNTER: LazyLock<Arc<Mutex<u64>>> = LazyLock::new(|| Arc::new(Mutex::new(0u64)));

pub(crate) struct ChunkSamplerDensityFunctionConverter<'a> {
    parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
}

impl<'a> VisitorImpl<'a> for ChunkSamplerDensityFunctionConverter<'a> {
    fn apply(&self, function: Arc<DensityFunction<'a>>) -> Arc<DensityFunction<'a>> {
        let sampler = self.parent_sampler.lock();
        let vertical_cell_count = sampler.vertical_cell_count;
        let horizontal_cell_count = sampler.horizontal_cell_count;

        let biome_start_x = sampler.start_biome_x;
        let biome_start_z = sampler.start_biome_z;
        let horizontal_biome_end = sampler.horizontal_biome_end as usize;

        let blender = sampler.blender;

        if let DensityFunction::Wrapper(wrapped) = function.as_ref() {
            match wrapped.wrapper() {
                WrapperType::Interpolated => Arc::new(DensityFunction::ChunkCacheInterpolator(
                    InterpolatorDensityFunction::new(
                        self.parent_sampler.clone(),
                        wrapped.wrapped(),
                        vertical_cell_count,
                        horizontal_cell_count,
                    ),
                )),
                WrapperType::Cache2D => Arc::new(DensityFunction::ChunkCache2DCache(
                    Cache2DDensityFunction::new(wrapped.wrapped()),
                )),
                WrapperType::CacheFlat => Arc::new(DensityFunction::ChunkCacheFlatCache(
                    FlatCacheDensityFunction::new(
                        self.parent_sampler.clone(),
                        wrapped.wrapped(),
                        biome_start_x,
                        biome_start_z,
                        horizontal_biome_end,
                        true,
                    ),
                )),
                WrapperType::CacheOnce => Arc::new(DensityFunction::ChunkCacheOnceCache(
                    CacheOnceDensityFunction::new(self.parent_sampler.clone(), wrapped.wrapped()),
                )),
                WrapperType::CacheCell => Arc::new(DensityFunction::ChunkCacheCellCache(
                    CellCacheDensityFunction::new(self.parent_sampler.clone(), wrapped.wrapped()),
                )),
            }
        } else if let Blender::None(_) = blender {
            if let DensityFunction::BlendAlpha(_) = function.as_ref() {
                let alpha_cache = {
                    sampler
                        .self_referential
                        .as_ref()
                        .unwrap()
                        .cached_blend_alpha_density_function
                        .clone()
                };
                Arc::new(DensityFunction::ChunkCacheFlatCache(alpha_cache))
            } else if let DensityFunction::BlendOffset(_) = function.as_ref() {
                let offset_cache = {
                    sampler
                        .self_referential
                        .as_ref()
                        .unwrap()
                        .cached_blend_offset_density_function
                        .clone()
                };
                Arc::new(DensityFunction::ChunkCacheFlatCache(offset_cache))
            } else {
                function
            }
        } else {
            function
        }
    }
}

struct ChunkNoiseSamplerSelfReferential<'a> {
    cached_blend_alpha_density_function: FlatCacheDensityFunction<'a>,
    cached_blend_offset_density_function: FlatCacheDensityFunction<'a>,
    block_state_sampler: BlockStateSampler<'a>,
    initial_density: Arc<DensityFunction<'a>>,
    aquifer_sampler: Arc<AquifierSampler>,
    interpolation_applier: Applier<'a>,
}

pub struct ChunkNoiseSampler<'a> {
    shape_config: GenerationShapeConfig,
    horizontal_cell_count: i32,
    vertical_cell_count: i32,
    minimum_cell_y: i32,
    start_cell_x: i32,
    start_cell_z: i32,
    start_biome_x: i32,
    start_biome_z: i32,
    interpolators: Vec<InterpolatorDensityFunctionWrapper<'a>>,
    caches: Vec<CellCacheDensityFunctionWrapper<'a>>,
    surface_height_estimate_cache: HashMap<u64, i32>,
    blender: &'a Blender,
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
    uuid: u64,
    visitor: Option<Arc<Visitor<'a>>>,
    self_referential: Option<ChunkNoiseSamplerSelfReferential<'a>>,
}

impl<'a> ChunkNoiseSampler<'a> {
    pub fn horizontal_cell_block_count(&self) -> i32 {
        self.horizontal_cell_block_count
    }

    pub fn vertical_cell_block_count(&self) -> i32 {
        self.vertical_cell_block_count
    }

    pub fn sample_start_density(&mut self) {
        if self.interpolating {
            panic!("interpolating twice");
        }

        self.interpolating = true;
        self.sample_unique = 0;
        self.sample_density(true, self.start_cell_x);
    }

    pub fn sample_end_density(&mut self, cell_x: i32) {
        self.sample_density(false, self.start_cell_x + cell_x + 1);
        self.start_block_x = (self.start_cell_x * cell_x) * self.horizontal_cell_block_count
    }

    fn sample_density(&mut self, start: bool, cell_x: i32) {
        self.start_block_x = cell_x * self.horizontal_cell_block_count;
        self.cell_block_x = 0;

        for i in 0..=self.horizontal_cell_count {
            let j = self.start_cell_z + i;
            self.start_cell_z = j * self.horizontal_cell_block_count;
            self.cell_block_z = 0;
            self.cache_unique += 1;

            for interpolator in self.interpolators.iter() {
                let mut wrapped = interpolator.wrapped.lock();
                let ds = &mut (if start {
                    &mut wrapped.start_buffer
                } else {
                    &mut wrapped.end_buffer
                })[i as usize];

                interpolator.fill(
                    ds,
                    &self
                        .self_referential
                        .as_ref()
                        .expect("call .init()")
                        .interpolation_applier,
                );
            }
        }

        self.cache_unique += 1;
    }

    fn init_visitor(me: Arc<Mutex<ChunkNoiseSampler<'a>>>) {
        let visitor = Visitor::ChunkSampler(ChunkSamplerDensityFunctionConverter {
            parent_sampler: me.clone(),
        });

        let mut sampler = me.lock();
        sampler.visitor = Some(Arc::new(visitor))
    }

    fn init(
        parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
        noise_router: &'a NoiseRouter<'a>,
        fluid_level: FluidLevelSampler,
        visitor: &Visitor<'a>,
    ) {
        let sampler = parent_sampler.lock();

        let mut cached_blend_alpha_density_function = FlatCacheDensityFunction::new(
            parent_sampler.clone(),
            Arc::new(DensityFunction::ChunkCacheBlendAlpha(
                BlendAlphaDensityFunction {
                    parent_sampler: parent_sampler.clone(),
                },
            )),
            sampler.start_biome_x,
            sampler.start_biome_z,
            sampler.horizontal_biome_end as usize,
            false,
        );

        let mut cached_blend_offset_density_function = FlatCacheDensityFunction::new(
            parent_sampler.clone(),
            Arc::new(DensityFunction::ChunkCacheBlendOffset(
                BlendOffsetDensityFunction {
                    parent_sampler: parent_sampler.clone(),
                },
            )),
            sampler.start_biome_x,
            sampler.start_biome_z,
            sampler.horizontal_biome_end as usize,
            false,
        );

        for i in 0..=(sampler.horizontal_biome_end as usize) {
            let j = sampler.start_biome_x + (i as i32);
            let k = biome_coords::to_block(j);
            for l in 0..=(sampler.horizontal_biome_end as usize) {
                let m = sampler.start_biome_z + (l as i32);
                let n = biome_coords::to_block(m);
                let result = sampler.blender.calculate(k, n);
                cached_blend_alpha_density_function.cache[i][l] = result.alpha();
                cached_blend_offset_density_function.cache[i][l] = result.offset();
            }
        }

        let new_router = noise_router.apply(visitor);
        let aquifer = Arc::new(AquifierSampler::SeaLevel(AquiferSeaLevel::new(fluid_level)));
        let final_density = new_router
            .final_densitiy
            .add(Arc::new(DensityFunction::Beardifyer(BeardifyerFunction {})));

        let function = DensityFunction::Wrapper(WrapperFunction::new(
            Arc::new(final_density),
            WrapperType::CacheCell,
        ));

        let block_sampler = BlockStateSampler::Aquifer(AquiferBlockStateSampler {
            aquifier: aquifer.clone(),
            function: function.apply(visitor),
        });

        let self_referring = ChunkNoiseSamplerSelfReferential {
            cached_blend_alpha_density_function,
            cached_blend_offset_density_function,
            aquifer_sampler: aquifer,
            block_state_sampler: block_sampler,
            initial_density: new_router.internal_density.clone(),
            interpolation_applier: Applier::Interpolation(InterpolationApplier {
                parent_sampler: parent_sampler.clone(),
            }),
        };
        let mut sampler = parent_sampler.lock();
        sampler.self_referential = Some(self_referring);
    }

    fn new(
        horizontal_cell_count: i32,
        start_block_x: i32,
        start_block_z: i32,
        shape: &GenerationShapeConfig,
        blender: &'a Blender,
    ) -> Self {
        let shape_config = shape.clone();
        let horizontal_cell_block_count = shape_config.horizontal_cell_block_count();
        let vertical_cell_block_count = shape_config.vertical_cell_block_count();
        let vertical_cell_count = shape_config.height() / vertical_cell_block_count;
        let minimum_cell_y = shape_config.min_y() / vertical_cell_block_count;
        let start_cell_x = start_block_x / horizontal_cell_block_count;
        let start_cell_z = start_block_z / horizontal_cell_block_count;
        let interpolators: Vec<InterpolatorDensityFunctionWrapper<'a>> = Vec::new();
        let caches: Vec<CellCacheDensityFunctionWrapper<'a>> = Vec::new();
        let start_biome_x = biome_coords::from_block(start_block_x);
        let start_biome_z = biome_coords::from_block(start_block_z);
        let horizontal_biome_end =
            biome_coords::from_block(horizontal_cell_count * horizontal_cell_block_count);

        let new_id = {
            let mut id = SAMPLER_ID_COUNTER.lock();
            let new_id = *id;
            *id += 1;
            new_id
        };

        Self {
            shape_config,
            horizontal_cell_block_count,
            vertical_cell_block_count,
            horizontal_cell_count,
            vertical_cell_count,
            minimum_cell_y,
            start_cell_x,
            start_cell_z,
            interpolators,
            caches,
            start_biome_x,
            start_biome_z,
            horizontal_biome_end,
            blender,
            surface_height_estimate_cache: HashMap::new(),
            last_blending_result: BlendResult::new(1f64, 0f64),
            last_blended_column_pos: CHUNK_MARKER,
            interpolating: false,
            sampling_for_caches: false,
            start_block_z: 0,
            start_block_x: 0,
            start_block_y: 0,
            cell_block_x: 0,
            cell_block_y: 0,
            cell_block_z: 0,
            sample_unique: 0,
            cache_unique: 0,
            uuid: new_id,
            index: 0,
            visitor: None,
            self_referential: None,
        }
    }

    fn calculate_blend_result(&mut self, block_x: i32, block_z: i32) -> BlendResult {
        let l = ChunkPos::new(block_x, block_z).to_long();
        if l != self.last_blended_column_pos {
            self.last_blended_column_pos = l;
            let blend_result = self.blender.calculate(block_x, block_z);
            self.last_blending_result = blend_result;
        }
        self.last_blending_result.clone()
    }
}

#[derive(Clone)]
pub(crate) struct ChunkNoiseSamplerWrapper<'a> {
    parent_sampler: Arc<Mutex<ChunkNoiseSampler<'a>>>,
}

impl<'a> ApplierImpl<'a> for ChunkNoiseSamplerWrapper<'a> {
    fn at(&self, index: usize) -> NoisePos<'a> {
        let index = index as i32;
        let mut sampler = self.parent_sampler.lock();
        let j = index % sampler.horizontal_cell_block_count;
        let k = index % sampler.horizontal_cell_block_count;
        let l = index % sampler.horizontal_cell_block_count;
        let m = sampler.vertical_cell_block_count - 1 - (k / sampler.horizontal_cell_block_count);

        sampler.cell_block_x = l;
        sampler.cell_block_y = m;
        sampler.cell_block_z = j;
        sampler.index = index;

        NoisePos::ChunkNoise(self.clone())
    }

    fn fill(&self, densities: &mut [f64], function: &DensityFunction<'a>) {
        let mut sampler = self.parent_sampler.lock();
        sampler.index = 0;
        for i in (0..=sampler.vertical_cell_block_count).rev() {
            sampler.cell_block_y = i;
            for j in 0..sampler.horizontal_cell_block_count {
                sampler.cell_block_x = j;
                for k in 0..sampler.horizontal_cell_block_count {
                    sampler.cell_block_z = k;
                    drop(sampler);
                    let sample = function.sample(&NoisePos::ChunkNoise(self.clone()));
                    sampler = self.parent_sampler.lock();
                    densities[sampler.index as usize] = sample;
                    sampler.index += 1;
                }
            }
        }
    }
}

impl<'a> NoisePosImpl for ChunkNoiseSamplerWrapper<'a> {
    fn z(&self) -> i32 {
        let sampler = self.parent_sampler.lock();
        sampler.start_block_z + sampler.cell_block_z
    }

    fn y(&self) -> i32 {
        let sampler = self.parent_sampler.lock();
        sampler.start_block_y + sampler.cell_block_y
    }

    fn x(&self) -> i32 {
        let sampler = self.parent_sampler.lock();
        sampler.start_block_x + sampler.cell_block_x
    }

    fn get_blender(&self) -> Blender {
        let sampler = self.parent_sampler.lock();
        sampler.blender.clone()
    }
}
