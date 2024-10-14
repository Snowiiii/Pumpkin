use std::{ops::Add, sync::Arc};

use num_traits::PrimInt;
use parking_lot::Mutex;

use crate::{
    biome::Biome, block::BlockId, chunk::ChunkData, height::HeightLimitViewImpl,
    world_gen::noise::chunk_sampler::ChunkNoiseSampler,
};

use super::{biome_coords, blender::data::BlendingData, heightmap::HeightMap};

#[derive(Clone, PartialEq, PartialOrd)]
pub enum GenerationState {
    Empty,
    StructureStart,
    StructureRef,
    Biome,
    Noise,
    Surface,
    Carver,
    Feature,
    InitLight,
    Light,
    Spawn,
    Full,
}

pub struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockPos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn z(&self) -> i32 {
        self.z
    }

    pub fn down(&self) -> Self {
        self.down_by(1)
    }

    pub fn down_by(&self, count: i32) -> Self {
        Self {
            x: self.x,
            y: self.y - count,
            z: self.z,
        }
    }

    pub fn up(&self) -> Self {
        self.up_by(1)
    }

    pub fn up_by(&self, count: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + count,
            z: self.z,
        }
    }

    pub fn north(&self) -> Self {
        self.north_by(1)
    }

    pub fn north_by(&self, count: i32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z - count,
        }
    }

    pub fn south(&self) -> Self {
        self.south_by(1)
    }

    pub fn south_by(&self, count: i32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z + count,
        }
    }

    pub fn west(&self) -> Self {
        self.west_by(1)
    }

    pub fn west_by(&self, count: i32) -> Self {
        Self {
            x: self.x - count,
            y: self.y,
            z: self.z,
        }
    }

    pub fn east(&self) -> Self {
        self.east_by(1)
    }

    pub fn east_by(&self, count: i32) -> Self {
        Self {
            x: self.x + count,
            y: self.y,
            z: self.z,
        }
    }
}

impl Add for BlockPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

pub enum HeightMapType {
    WorldGenSurface,
    WorldSurface,
    WorldGenOceanFloor,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves,
}

pub struct VoxelShape {}

impl VoxelShape {
    pub fn is_empty(&self) -> bool {
        unimplemented!()
    }
}

pub struct BlockState {}

impl BlockState {
    pub fn block(&self) -> BlockId {
        unimplemented!()
    }

    pub fn is_air(&self) -> bool {
        unimplemented!()
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        unimplemented!()
    }

    pub fn is_block(&self, id: BlockId) -> bool {
        unimplemented!()
    }

    pub fn collision_shape(&self, chunk: &Chunk, pos: &BlockPos) -> VoxelShape {
        unimplemented!()
    }
}

pub const CHUNK_MARKER: u64 = ChunkPos {
    x: 1875066,
    z: 1875066,
}
.to_long();

pub struct ChunkPos {
    x: i32,
    z: i32,
}

impl ChunkPos {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    pub const fn to_long(&self) -> u64 {
        (self.x as u64 & 4294967295u64) | ((self.z as u64 & 4294967295u64) << 32)
    }

    pub fn packed_x(pos: u64) -> i32 {
        (pos & 4294967295u64) as i32
    }

    pub fn packed_z(pos: u64) -> i32 {
        (pos.unsigned_shr(32) & 4294967295u64) as i32
    }

    pub fn get_start_x(&self) -> i32 {
        self.x << 4
    }

    pub fn get_start_z(&self) -> i32 {
        self.z << 4
    }
}

pub struct Chunk {
    pos: ChunkPos,
    data: ChunkData,
    state: GenerationState,
}

impl Chunk {
    pub fn get_block_state(&self, pos: &BlockPos) -> BlockState {
        unimplemented!()
    }

    pub fn sample_height_map(&self, map: HeightMapType, x: i32, y: i32) -> Option<i32> {
        unimplemented!()
    }

    pub fn biome_for_noise_gen(&self, biome_x: i32, biome_y: i32, biome_z: i32) -> Biome {
        unimplemented!()
    }

    pub fn blending_data(&self) -> Option<BlendingData> {
        unimplemented!()
    }

    pub fn status(&self) -> GenerationState {
        self.state.clone()
    }

    pub fn get_or_create_noise_sampler(&self) -> Arc<Mutex<ChunkNoiseSampler>> {
        unimplemented!()
    }

    pub fn get_height_map(&self, map: HeightMapType) -> &HeightMap {
        unimplemented!()
    }

    pub fn pos(&self) -> &ChunkPos {
        &self.pos
    }
}

impl HeightLimitViewImpl for Chunk {
    fn bottom_y(&self) -> i32 {
        unimplemented!()
    }

    fn height(&self) -> i32 {
        unimplemented!()
    }
}

#[derive(Clone)]
pub struct GenerationShapeConfig {
    y_min: i32,
    height: i32,
    horizontal: i32,
    vertical: i32,
}

//Bits avaliable to encode y-pos
pub const SIZE_BITS_Y: i32 = 12;
pub const MAX_HEIGHT: i32 = (1 << SIZE_BITS_Y) - 32;
pub const MAX_COLUMN_HEIGHT: i32 = (MAX_HEIGHT >> 1) - 1;
pub const MIN_HEIGHT: i32 = MAX_COLUMN_HEIGHT - MAX_HEIGHT + 1;

impl GenerationShapeConfig {
    fn new(y_min: i32, height: i32, horizontal: i32, vertical: i32) -> Self {
        if (y_min + height) > (MAX_COLUMN_HEIGHT + 1) {
            panic!("Cannot be higher than max column height");
        } else if height % 16 != 0 {
            panic!("Height must be a multiple of 16");
        } else if y_min % 16 != 0 {
            panic!("Y min must be a multiple of 16");
        }
        Self {
            y_min,
            height,
            horizontal,
            vertical,
        }
    }

    pub fn trim_height(&self, view: &impl HeightLimitViewImpl) -> Self {
        let i = self.y_min.max(view.bottom_y());
        let j = (self.y_min + self.height).min(view.top_y()) - i;
        Self {
            y_min: i,
            height: j,
            horizontal: self.horizontal,
            vertical: self.vertical,
        }
    }

    pub fn min_y(&self) -> i32 {
        self.y_min
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn vertical_cell_block_count(&self) -> i32 {
        biome_coords::to_block(self.vertical)
    }

    pub fn horizontal_cell_block_count(&self) -> i32 {
        biome_coords::to_block(self.horizontal)
    }
}

pub mod shape_configs {
    use super::GenerationShapeConfig;

    pub const surface_config: GenerationShapeConfig = GenerationShapeConfig {
        y_min: -64,
        height: 384,
        horizontal: 1,
        vertical: 2,
    };
}
