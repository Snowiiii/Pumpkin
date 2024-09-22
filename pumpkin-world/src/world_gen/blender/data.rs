use crate::{
    self as pumpkin_world,
    biome::Biome,
    block::BlockId,
    height::{HeightLimitView, HeightLimitViewImpl, StandardHeightLimitView},
    world_gen::{
        chunk::{BlockPos, Chunk, GenerationState, HeightMapType},
        Direction,
    },
};
use num_traits::PrimInt;
use pumpkin_macros::block_id;

pub const SURFACE_BLOCKS: [BlockId; 11] = [
    block_id!("minecraft:podzol"),
    block_id!("minecraft:gravel"),
    block_id!("minecraft:grass_block"),
    block_id!("minecraft:stone"),
    block_id!("minecraft:coarse_dirt"),
    block_id!("minecraft:sand"),
    block_id!("minecraft:red_sand"),
    block_id!("minecraft:mycelium"),
    block_id!("minecraft:snow_block"),
    block_id!("minecraft:terracotta"),
    block_id!("minecraft:dirt"),
];

const BIOMES_PER_CHUNK: usize = 16 >> 2;
const LAST_CHUNK_BIOME_INDEX: usize = BIOMES_PER_CHUNK - 1;
const CHUNK_BIOME_END_INDEX: usize = BIOMES_PER_CHUNK;
const NORTH_WEST_END_INDEX: usize = 2 * LAST_CHUNK_BIOME_INDEX + 1;
const SOUTH_EAST_END_INDEX_PART: usize = 2 * CHUNK_BIOME_END_INDEX + 1;
const HORIZONTAL_BIOME_COUNT: usize = NORTH_WEST_END_INDEX + SOUTH_EAST_END_INDEX_PART;

pub struct BlendingData {
    pub(crate) height_limit: HeightLimitView,
    pub(crate) surface_heights: Box<[f64; HORIZONTAL_BIOME_COUNT]>,
    pub(crate) biomes: Box<[Option<Vec<Biome>>; HORIZONTAL_BIOME_COUNT]>,
    pub(crate) collidable_block_densities: Box<[Option<Vec<f64>>; HORIZONTAL_BIOME_COUNT]>,
    initialized: bool,
}

impl BlendingData {
    pub fn get_blending_data(chunk: &Chunk, _chunk_x: i32, _chunk_z: i32) -> Option<Self> {
        // TODO: We currently assume all chunks have new noise. valid assumption?
        if let Some(mut data) = chunk.blending_data() {
            if chunk.status() >= GenerationState::Biome {
                data.init_chunk_blending_data(
                    chunk,
                    &[
                        Direction::North,
                        Direction::NorthWest,
                        Direction::West,
                        Direction::SouthWest,
                        Direction::South,
                        Direction::SouthEast,
                        Direction::East,
                        Direction::NorthEast,
                    ],
                );
                Some(data)
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline]
    fn method_39355(i: usize) -> usize {
        i & !(i.unsigned_shr(31))
    }

    pub(crate) fn x(index: usize) -> usize {
        if index < NORTH_WEST_END_INDEX {
            Self::method_39355(LAST_CHUNK_BIOME_INDEX - index)
        } else {
            let i = index - NORTH_WEST_END_INDEX;
            CHUNK_BIOME_END_INDEX - Self::method_39355(CHUNK_BIOME_END_INDEX - i)
        }
    }

    pub(crate) fn z(index: usize) -> usize {
        if index < NORTH_WEST_END_INDEX {
            Self::method_39355(index - LAST_CHUNK_BIOME_INDEX)
        } else {
            let i = index - NORTH_WEST_END_INDEX;
            CHUNK_BIOME_END_INDEX - Self::method_39355(i - CHUNK_BIOME_END_INDEX)
        }
    }

    pub fn height(&self, biome_x: usize, _biome_y: usize, biome_z: usize) -> f64 {
        if biome_x == CHUNK_BIOME_END_INDEX || biome_z == CHUNK_BIOME_END_INDEX {
            self.surface_heights[Self::south_east_index(biome_x, biome_z)]
        } else if biome_x != 0 && biome_z != 0 {
            f64::MAX
        } else {
            self.surface_heights[Self::north_west_index(biome_x, biome_z)]
        }
    }

    fn collidable_block_density_from_column(
        &self,
        column: Option<&[f64]>,
        half_section_y: i32,
    ) -> f64 {
        if let Some(column) = column {
            let i = self.half_section_height(half_section_y);
            if (i >= 0) && ((i as usize) < column.len()) {
                column[i as usize] * 0.1f64
            } else {
                f64::MAX
            }
        } else {
            f64::MAX
        }
    }

    pub fn collidable_block_density(
        &self,
        biome_x: usize,
        half_section_y: i32,
        biome_z: usize,
    ) -> f64 {
        if half_section_y == self.bottom_half_section_y() {
            0.1f64
        } else if biome_x == CHUNK_BIOME_END_INDEX || biome_z == CHUNK_BIOME_END_INDEX {
            self.collidable_block_density_from_column(
                self.collidable_block_densities[Self::south_east_index(biome_x, biome_z)]
                    .as_deref(),
                half_section_y,
            )
        } else if biome_x != 0 && biome_z != 0 {
            f64::MAX
        } else {
            self.collidable_block_density_from_column(
                self.collidable_block_densities[Self::north_west_index(biome_x, biome_z)]
                    .as_deref(),
                half_section_y,
            )
        }
    }

    fn new(bottom_section_y: i32, top_section_y: i32, heights: Option<&[f64]>) -> Self {
        let heights = match heights {
            Some(heights) => {
                let mut owned_heights = [0f64; HORIZONTAL_BIOME_COUNT];
                assert!(
                    heights.len() == HORIZONTAL_BIOME_COUNT,
                    "Heights needs to be the right length"
                );
                owned_heights
                    .iter_mut()
                    .zip(heights)
                    .for_each(|(new, old)| {
                        *new = *old;
                    });
                owned_heights
            }
            None => [f64::MAX; HORIZONTAL_BIOME_COUNT],
        };

        let collidable_block_densities: [Option<Vec<f64>>; HORIZONTAL_BIOME_COUNT] =
            [const { None }; HORIZONTAL_BIOME_COUNT];
        let biomes: [Option<Vec<Biome>>; HORIZONTAL_BIOME_COUNT] =
            [const { None }; HORIZONTAL_BIOME_COUNT];

        let i = bottom_section_y << 4;
        let j = (top_section_y << 4) - i;
        let height_limit = HeightLimitView::Standard(StandardHeightLimitView::new(i, j));

        Self {
            height_limit,
            surface_heights: Box::new(heights.clone()),
            biomes: Box::new(biomes),
            collidable_block_densities: Box::new(collidable_block_densities),
            initialized: false,
        }
    }

    pub fn vertical_half_section_count(&self) -> i32 {
        self.height_limit.vertical_section_count() * 2
    }

    fn collidable_and_not_tree(chunk: &Chunk, pos: &BlockPos) -> bool {
        let state = chunk.get_block_state(pos);
        if state.is_air()
            || state.has_tag("leaves")
            || state.has_tag("logs")
            || state.is_block(block_id!("minecraft:brown_mushroom_block"))
            || state.is_block(block_id!("minecraft:red_mushroom_block"))
        {
            false
        } else {
            state.collision_shape(chunk, pos).is_empty()
        }
    }

    fn above_collidable_block_value(chunk: &Chunk, pos: &BlockPos) -> (f64, BlockPos) {
        let pos = pos.down();
        let val = if Self::collidable_and_not_tree(chunk, &pos) {
            1f64
        } else {
            -1f64
        };
        (val, pos)
    }

    fn north_west_index(biome_x: usize, biome_z: usize) -> usize {
        LAST_CHUNK_BIOME_INDEX + biome_x + biome_z
    }

    fn south_east_index(biome_x: usize, biome_z: usize) -> usize {
        NORTH_WEST_END_INDEX + biome_x + CHUNK_BIOME_END_INDEX - biome_z
    }

    fn init_chunk_blending_data(&mut self, chunk: &Chunk, directions: &[Direction]) {
        if !self.initialized {
            if directions.contains(&Direction::North)
                || directions.contains(&Direction::West)
                || directions.contains(&Direction::NorthWest)
            {
                self.init_block_column(Self::north_west_index(0, 0), chunk, 0, 0);
            }

            if directions.contains(&Direction::North) {
                for i in 1..BIOMES_PER_CHUNK {
                    self.init_block_column(Self::north_west_index(i, 0), chunk, (4 * i) as i32, 0);
                }
            }

            if directions.contains(&Direction::West) {
                for i in 1..BIOMES_PER_CHUNK {
                    self.init_block_column(Self::north_west_index(0, i), chunk, 0, (4 * i) as i32);
                }
            }

            if directions.contains(&Direction::East) {
                for i in 1..BIOMES_PER_CHUNK {
                    self.init_block_column(
                        Self::south_east_index(CHUNK_BIOME_END_INDEX, i),
                        chunk,
                        15,
                        (4 * i) as i32,
                    );
                }
            }

            if directions.contains(&Direction::South) {
                for i in 1..BIOMES_PER_CHUNK {
                    self.init_block_column(
                        Self::south_east_index(i, CHUNK_BIOME_END_INDEX),
                        chunk,
                        (4 * i) as i32,
                        15,
                    );
                }
            }

            if directions.contains(&Direction::East) && directions.contains(&Direction::NorthEast) {
                self.init_block_column(
                    Self::south_east_index(CHUNK_BIOME_END_INDEX, 0),
                    chunk,
                    15,
                    0,
                );
            }

            if directions.contains(&Direction::East)
                && directions.contains(&Direction::South)
                && directions.contains(&Direction::SouthEast)
            {
                self.init_block_column(
                    Self::south_east_index(CHUNK_BIOME_END_INDEX, CHUNK_BIOME_END_INDEX),
                    chunk,
                    15,
                    15,
                );
            }

            self.initialized = true;
        }
    }

    fn collidable_block_density_below(chunk: &Chunk, pos: &BlockPos) -> (f64, BlockPos) {
        let (val, new_pos) = Self::above_collidable_block_value(chunk, &pos);
        let mut d = val;
        let mut pos = new_pos;

        for _ in 0..6 {
            let (val, new_pos) = Self::above_collidable_block_value(chunk, &pos);
            d += val;
            pos = new_pos;
        }

        (d, pos)
    }

    pub fn bottom_half_section_y(&self) -> i32 {
        self.height_limit.bottom_section_coord() * 2
    }

    fn half_section_height(&self, section_y: i32) -> i32 {
        section_y - (self.bottom_half_section_y() + 1)
    }

    fn init_block_column(&mut self, index: usize, chunk: &Chunk, chunk_x: i32, chunk_z: i32) {
        if self.surface_heights[index] == f64::MAX {
            self.surface_heights[index] = self.surface_block_y(chunk, chunk_x, chunk_z) as f64;
        }
        self.collidable_block_densities[index] = Some(self.collidable_block_density_column(
            chunk,
            chunk_x,
            chunk_z,
            self.surface_heights[index].floor() as i32,
        ));
        self.biomes[index] = Some(self.vertical_biome_sections(chunk, chunk_x, chunk_z));
    }

    fn vertical_biome_sections(&self, chunk: &Chunk, block_x: i32, block_z: i32) -> Vec<Biome> {
        (0..self.vertical_biome_count())
            .map(|i| {
                let j = i + (self.height_limit.bottom_y() >> 2);
                chunk.biome_for_noise_gen(block_x >> 2, j, block_z >> 2)
            })
            .collect()
    }

    fn vertical_biome_count(&self) -> i32 {
        self.height_limit.vertical_section_count() << 2
    }

    fn collidable_block_density_column(
        &self,
        chunk: &Chunk,
        chunk_x: i32,
        chunk_z: i32,
        height: i32,
    ) -> Vec<f64> {
        let mut ds: Vec<f64> = (0..self.vertical_half_section_count())
            .map(|_| -1f64)
            .collect();

        let pos = BlockPos::new(chunk_x, self.height_limit.top_y(), chunk_z);
        let (mut d, mut pos) = Self::collidable_block_density_below(chunk, &pos);

        for i in (0..=(ds.len() - 2)).rev() {
            let (e, local_pos) = Self::above_collidable_block_value(chunk, &pos);
            let (f, local_pos) = Self::collidable_block_density_below(chunk, &local_pos);
            ds[i] = (d + e + f) / 15f64;
            d = f;
            pos = local_pos;
        }

        let i = self.half_section_height(height / 8);
        if i >= 0 && (i as usize) < (ds.len() - 1) {
            let e = (height as f64 + 0.5f64) % 8f64 / 8f64;
            let f = (1f64 - e) / e;
            let g = f.max(1f64) * 0.25f64;
            ds[(i + 1) as usize] = -f / g;
            ds[i as usize] = 1f64 / g;
        }

        ds
    }

    fn surface_block_y(&self, chunk: &Chunk, block_x: i32, block_z: i32) -> i32 {
        let i = if let Some(value) =
            chunk.sample_height_map(HeightMapType::WorldGenSurface, block_x, block_z)
        {
            value
        } else {
            self.height_limit.top_y()
        };

        let j = self.height_limit.bottom_y();

        for height in (j..=i).rev() {
            if SURFACE_BLOCKS.contains(
                &chunk
                    .get_block_state(&BlockPos::new(block_x, height, block_z))
                    .block(),
            ) {
                return height;
            }
        }

        j
    }
}
