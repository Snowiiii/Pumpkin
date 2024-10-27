use std::{collections::VecDeque, time::Instant};

use pumpkin_core::math::vector2::Vector2;

use crate::{
    block::{Block, BlockCategory, BlockId},
    chunk::{self, ChunkBlocks},
};

fn div_16_floor(y: i32) -> i32 {
    if y >= 0 {
        y / 16
    } else {
        (y / 16) - 1
    }
}

#[derive(Debug, Clone)]
struct Coordinates {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkCoordinates {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone)]
struct ChunkRelativeCoordinates {
    x: u8,
    y: i32,
    z: u8,
}

#[derive(Debug, Clone)]
struct SubChunkRelativeCoordinates {
    x: u8,
    y: u8,
    z: u8,
}

impl From<Coordinates> for ChunkRelativeCoordinates {
    fn from(value: Coordinates) -> Self {
        ChunkRelativeCoordinates {
            x: value.x.rem_euclid(16) as u8,
            y: value.y,
            z: value.z.rem_euclid(16) as u8,
        }
    }
}

impl From<Coordinates> for SubChunkRelativeCoordinates {
    fn from(value: Coordinates) -> Self {
        SubChunkRelativeCoordinates {
            x: value.x.rem_euclid(16) as u8,
            y: value.y.rem_euclid(16) as u8,
            z: value.z.rem_euclid(16) as u8,
        }
    }
}

impl From<ChunkRelativeCoordinates> for SubChunkRelativeCoordinates {
    fn from(value: ChunkRelativeCoordinates) -> Self {
        SubChunkRelativeCoordinates {
            x: value.x,
            y: value.y.rem_euclid(16) as u8,
            z: value.z,
        }
    }
}

impl Coordinates {
    fn chunk_coordinates(&self) -> ChunkCoordinates {
        ChunkCoordinates {
            x: div_16_floor(self.x),
            z: div_16_floor(self.z),
        }
    }
}
impl ChunkRelativeCoordinates {
    fn with_chunk_coordinates(&self, chunk_coordinates: &ChunkCoordinates) -> Coordinates {
        Coordinates {
            x: chunk_coordinates.x * 16 + self.x as i32,
            y: self.y,
            z: chunk_coordinates.z * 16 + self.z as i32,
        }
    }
}

struct LightChangeQueueItem {
    coordinates: ChunkRelativeCoordinates,
    level: u8,
}

const MAX_LIGHT_SUBCHUNK: i32 = 20;
const MIN_LIGHT_SUBCHUNK: i32 = -5;
const TOTAL_LIGHT_SUBCHUNKS: usize = (MAX_LIGHT_SUBCHUNK - MIN_LIGHT_SUBCHUNK) as usize + 1;
const MAX_BLOCK_SUBCHUNK: i32 = 19;
const MIN_BLOCK_SUBCHUNK: i32 = -4;
const TOTAL_BLOCK_SUBCHUNKS: usize = (MAX_BLOCK_SUBCHUNK - MIN_BLOCK_SUBCHUNK) as usize + 1;

const CHUNK_AREA: usize = 16 * 16;
const SUBCHUNK_VOLUME: usize = CHUNK_AREA * 16;

#[derive(Debug)]
pub struct SubChunkLightData(pub Box<[u8; SUBCHUNK_VOLUME / 2]>);

impl SubChunkLightData {
    fn get_index(coordinates: SubChunkRelativeCoordinates) -> (usize, bool) {
        let full_index = ((coordinates.y as usize) << 8)
            | ((coordinates.z as usize) << 4)
            | (coordinates.x as usize);
        let index = full_index / 2;
        let has_remainder = (full_index & 1) == 1;

        (index, has_remainder)
    }

    fn get_level_at(&self, coordinates: impl Into<SubChunkRelativeCoordinates>) -> u8 {
        let coordinates: SubChunkRelativeCoordinates = coordinates.into();

        let (index, has_remainder) = Self::get_index(coordinates);

        if has_remainder {
            (self.0[index] & 0xF0) >> 4
        } else {
            self.0[index] & 0x0F
        }
    }

    fn set_level_at(&mut self, coordinates: impl Into<SubChunkRelativeCoordinates>, level: u8) {
        let coordinates: SubChunkRelativeCoordinates = coordinates.into();

        let (index, has_remainder) = Self::get_index(coordinates);

        if has_remainder {
            self.0[index] = (self.0[index] & 0x0F) | (level << 4);
        } else {
            self.0[index] = (self.0[index] & 0xF0) | level;
        }
    }
}

#[derive(Debug)]
pub enum SubChunkLighting {
    Uninitialized,
    Initialized(SubChunkLightData),
}

impl SubChunkLighting {
    fn initialized() -> Self {
        Self::Initialized(SubChunkLightData {
            0: Box::new([0; SUBCHUNK_VOLUME / 2]),
        })
    }
}

pub struct ChunkLightData {
    pub subchunks: [SubChunkLighting; TOTAL_LIGHT_SUBCHUNKS],
    increase_queue: VecDeque<LightChangeQueueItem>,
    chunk_coordinates: ChunkCoordinates,
}

impl ChunkLightData {
    fn get_subchunk(&self, y: i32) -> &SubChunkLighting {
        let index = (y - MIN_LIGHT_SUBCHUNK) as usize;
        &self.subchunks[index]
    }

    fn get_subchunk_mut(&mut self, y: i32) -> &mut SubChunkLighting {
        let index = (y - MIN_LIGHT_SUBCHUNK) as usize;
        &mut self.subchunks[index]
    }

    fn set_subchunk(&mut self, y: i32, lighting: SubChunkLighting) {
        let index = (y - MIN_LIGHT_SUBCHUNK) as usize;
        self.subchunks[index] = lighting
    }
}

enum Direction {
    XPos,
    XNeg,
    YPos,
    YNeg,
    ZPos,
    ZNeg,
}

impl Direction {
    const VALUES: [Self; 6] = [
        Self::XPos,
        Self::XNeg,
        Self::YPos,
        Self::YNeg,
        Self::ZPos,
        Self::ZNeg,
    ];
}

enum InDirection {
    Valid(Coordinates),
    Invalid,
}

impl Coordinates {
    fn in_direction(&self, direction: Direction) -> InDirection {
        match direction {
            Direction::XPos => InDirection::Valid(Self {
                x: self.x + 1,
                y: self.y,
                z: self.z,
            }),
            Direction::XNeg => InDirection::Valid(Self {
                x: self.x - 1,
                y: self.y,
                z: self.z,
            }),
            Direction::YPos => {
                if self.y + 1 >= MAX_LIGHT_SUBCHUNK * 16 {
                    InDirection::Invalid
                } else {
                    InDirection::Valid(Self {
                        x: self.x,
                        y: self.y + 1,
                        z: self.z,
                    })
                }
            }
            Direction::YNeg => {
                if self.y <= MIN_LIGHT_SUBCHUNK * 16 {
                    InDirection::Invalid
                } else {
                    InDirection::Valid(Self {
                        x: self.x,
                        y: self.y - 1,
                        z: self.z,
                    })
                }
            }
            Direction::ZPos => InDirection::Valid(Self {
                x: self.x,
                y: self.y,
                z: self.z + 1,
            }),
            Direction::ZNeg => InDirection::Valid(Self {
                x: self.x,
                y: self.y,
                z: self.z - 1,
            }),
        }
    }
}

impl ChunkLightData {
    fn block_light_filtering(block: BlockId) -> u8 {
        if block.is_air() {
            0
        } else {
            15
        }
    }

    fn add_to_increase_queue(&mut self, coordinates: ChunkRelativeCoordinates, level: u8) {
        self.increase_queue
            .push_back(LightChangeQueueItem { level, coordinates });
    }

    fn subchunk_empty(blocks: &ChunkBlocks, i: i32) -> bool {
        if i < MIN_BLOCK_SUBCHUNK || i > MAX_BLOCK_SUBCHUNK {
            return true;
        }

        for y in 0..16 {
            for x in 0u8..16 {
                for z in 0u8..16 {
                    let y = y + i * 16;
                    let block =
                        blocks.get_block(crate::coordinates::ChunkRelativeBlockCoordinates {
                            x: x.into(),
                            y: y.into(),
                            z: z.into(),
                        });

                    if !block.is_air() {
                        return false;
                    }
                }
            }
        }
        true
    }

    fn directions(coordinates: Coordinates) -> [InDirection; 6] {
        Direction::VALUES.map(|direction| coordinates.in_direction(direction))
    }

    fn get_light_level(&self, coordinates: ChunkRelativeCoordinates) -> u8 {
        let subchunk_y = std::cmp::max(div_16_floor(coordinates.y), MIN_LIGHT_SUBCHUNK);

        if subchunk_y > MAX_LIGHT_SUBCHUNK {
            return 15;
        }

        let subchunk = self.get_subchunk(subchunk_y);
        match subchunk {
            SubChunkLighting::Uninitialized => self.get_light_level(ChunkRelativeCoordinates {
                x: coordinates.x,
                y: (subchunk_y + 1) * 16,
                z: coordinates.z,
            }),
            SubChunkLighting::Initialized(light_data) => light_data.get_level_at(coordinates),
        }
    }

    fn set_light_level(&mut self, coordinates: ChunkRelativeCoordinates, level: u8) {
        let subchunk_y = div_16_floor(coordinates.y);

        let subchunk = self.get_subchunk_mut(subchunk_y);
        match subchunk {
            SubChunkLighting::Uninitialized => (),
            SubChunkLighting::Initialized(light_data) => {
                light_data.set_level_at(coordinates, level)
            }
        }
    }

    fn propogate_increase(&mut self, blocks: &ChunkBlocks) {
        while let Some(item) = self.increase_queue.pop_front() {
            let world_coordinates = item
                .coordinates
                .with_chunk_coordinates(&self.chunk_coordinates);
            for direction in Self::directions(world_coordinates.clone()) {
                let InDirection::Valid(neighbor) = direction else {
                    continue;
                };

                let chunk_coordinates = neighbor.chunk_coordinates();

                if self.chunk_coordinates != chunk_coordinates {
                    // TODO: Propogate over a chunk boundary
                    continue;
                }

                let chunk_relative_coordinates: ChunkRelativeCoordinates = neighbor.clone().into();
                let current_level = self.get_light_level(chunk_relative_coordinates.clone());

                let target_level = item.level - 1;

                if current_level >= target_level {
                    continue;
                }

                if chunk_relative_coordinates.y >= MIN_BLOCK_SUBCHUNK * 16
                    && chunk_relative_coordinates.y < MAX_BLOCK_SUBCHUNK * 16
                {
                    let block =
                        blocks.get_block(crate::coordinates::ChunkRelativeBlockCoordinates {
                            x: chunk_relative_coordinates.x.into(),
                            y: chunk_relative_coordinates.y.into(),
                            z: chunk_relative_coordinates.z.into(),
                        });

                    // TODO: Handle other transparent blocks
                    if !block.is_air() {
                        continue;
                    }
                }

                self.set_light_level(chunk_relative_coordinates.clone(), target_level);
                self.add_to_increase_queue(chunk_relative_coordinates, target_level);
            }
        }
    }

    pub fn initialize(chunk_coordinates: Vector2<i32>, blocks: &ChunkBlocks) -> Self {
        const BITS_PER_ENTRY: usize = 9;
        const ENTRIES_PER_LONG: usize = 64 / BITS_PER_ENTRY;

        let mut column_heights = Vec::new();

        for value in blocks.heightmap.world_surface.iter() {
            for i in 0..ENTRIES_PER_LONG {
                // Get last 9 bits
                let foo = (value >> (BITS_PER_ENTRY * i)) & 0b111111111;
                column_heights.push(foo as i32);
            }
        }

        // -65 because the column heights tracks total height, 0 should be -65 as if there is no
        // height it should be below the height of the world
        let highest_block_y = column_heights.iter().max().unwrap_or(&0) - 65;

        let mut chunk_light_data = Self {
            chunk_coordinates: ChunkCoordinates {
                x: chunk_coordinates.x,
                z: chunk_coordinates.z,
            },
            subchunks: [const { SubChunkLighting::Uninitialized }; TOTAL_LIGHT_SUBCHUNKS],
            increase_queue: VecDeque::new(),
        };

        let subchunks_with_blocks: Vec<_> = (MIN_LIGHT_SUBCHUNK..=MAX_LIGHT_SUBCHUNK)
            .map(|i| i * 16 <= highest_block_y && !Self::subchunk_empty(blocks, i))
            .collect();

        for subchunk_y in MIN_LIGHT_SUBCHUNK..=MAX_LIGHT_SUBCHUNK {
            let i = (subchunk_y - MIN_LIGHT_SUBCHUNK) as usize;
            if *subchunks_with_blocks.get(i).unwrap_or(&false)
                || *subchunks_with_blocks.get(i + 1).unwrap_or(&false)
                || *subchunks_with_blocks.get(i - 1).unwrap_or(&false)
            {
                chunk_light_data.set_subchunk(subchunk_y, SubChunkLighting::initialized());
            }
        }

        // Sky Light Columns
        for z in 0u8..16 {
            for x in 0u8..16 {
                let column_index = (z as usize) * 16 + x as usize;
                let column_height = column_heights[column_index];

                let mut light_level = 15;
                // Start from the top down
                'column: for (subchunk_index, subchunk) in
                    chunk_light_data.subchunks.iter_mut().enumerate().rev()
                {
                    let subchunk_y = subchunk_index as i32 + MIN_LIGHT_SUBCHUNK;
                    let SubChunkLighting::Initialized(lighting_data) = subchunk else {
                        continue;
                    };

                    // First block is guaranteed to not be filled because if the entire world is
                    // filled, the first subchunk will be above the world
                    for y in (0..16).rev() {
                        lighting_data
                            .set_level_at(SubChunkRelativeCoordinates { x, y, z }, light_level);

                        let current_block_y = (subchunk_y * 16) + y as i32;
                        if current_block_y > (column_height - 64) {
                            // Is guaranteed that the next block is air
                            continue;
                        }

                        let next_block_y = (subchunk_y * 16) + (y as i32) - 1;

                        let light_reduction = if next_block_y < MAX_BLOCK_SUBCHUNK * 16
                            && next_block_y >= MIN_BLOCK_SUBCHUNK * 16
                        {
                            let next_block = blocks.get_block(
                                crate::coordinates::ChunkRelativeBlockCoordinates {
                                    x: x.into(),
                                    y: next_block_y.into(),
                                    z: z.into(),
                                },
                            );

                            Self::block_light_filtering(next_block)
                        } else {
                            0
                        };

                        let next_light_level = std::cmp::max(0, light_level - light_reduction);

                        if next_light_level == 0 {
                            chunk_light_data.add_to_increase_queue(
                                ChunkRelativeCoordinates {
                                    x,
                                    y: current_block_y,
                                    z,
                                },
                                light_level,
                            );
                            break 'column;
                        }

                        light_level = next_light_level;
                    }
                }
            }
        }

        chunk_light_data.propogate_increase(&blocks);

        chunk_light_data
    }

    pub fn packet_data(&self) -> (i64, i64, Vec<&Box<[u8; 2048]>>) {
        let mut empty_mask = 0;
        let mut set_mask = 0;
        let mut things = Vec::new();

        for (i, subchunk) in self.subchunks.iter().enumerate() {
            match subchunk {
                SubChunkLighting::Uninitialized => {}
                SubChunkLighting::Initialized(light_data) => {
                    // Add to empty
                    empty_mask |= 1 << i;
                    for level in light_data.0.iter() {
                        if *level != 0 {
                            things.push(&light_data.0);
                            set_mask |= 1 << i;
                            // Remove from empty
                            empty_mask ^= 1 << i;
                            break;
                        }
                    }
                }
            }
        }

        (set_mask, empty_mask, things)
    }
}

// TODO: Chunk Lighting
// This is going to requie access to the rest of the chunks, not sure on how to do this atm.
// Upon initial chunk initialization, pull values from edges of chunk
// Upon lighting propogation, queue changes into the new chunk as they leave the border. If those
// aren't initialized yet, don't do anything, the value will be pulled upon that chunk initializing.
//
// TODO: Subchunk Initialization
// Only subchunks that are adjacent (including diagonals) to a subchunk with blocks should be
// initialized.
// Subchunks will have to be cleared when there is no longer a nearby block. Either checking the
// nearby chunks again, or some kind of map that has the current non empty chunks
