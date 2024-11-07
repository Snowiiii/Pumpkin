use std::collections::HashMap;
use std::cmp::max;

use fastnbt::LongArray;
use pumpkin_core::math::vector2::Vector2;
use rle_vec::RleVec;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    block::BlockState,
    coordinates::{ChunkRelativeBlockCoordinates, Height},
    level::SaveFile,
    WORLD_HEIGHT,
};

pub mod anvil;

const CHUNK_AREA: usize = 16 * 16;
const SUBCHUNK_VOLUME: usize = CHUNK_AREA * 16;
const CHUNK_VOLUME: usize = CHUNK_AREA * WORLD_HEIGHT;

pub trait ChunkReader: Sync + Send {
    fn read_chunk(
        &self,
        save_file: &SaveFile,
        at: &Vector2<i32>,
    ) -> Result<ChunkData, ChunkReadingError>;
}

#[derive(Error, Debug)]
pub enum ChunkReadingError {
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Region is invalid")]
    RegionIsInvalid,
    #[error("Compression error {0}")]
    Compression(CompressionError),
    #[error("Tried to read chunk which does not exist")]
    ChunkNotExist,
    #[error("Failed to parse Chunk from bytes: {0}")]
    ParsingError(ChunkParsingError),
}

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Compression scheme not recognised")]
    UnknownCompression,
    #[error("Error while working with zlib compression: {0}")]
    ZlibError(std::io::Error),
    #[error("Error while working with Gzip compression: {0}")]
    GZipError(std::io::Error),
    #[error("Error while working with LZ4 compression: {0}")]
    LZ4Error(std::io::Error),
}

#[derive(Debug)]
pub struct ChunkData {
    pub blocks: ChunkBlocks,
    /// See `https://minecraft.fandom.com/wiki/Heightmap` for more info
    pub heightmap: ChunkHeightmaps,
    pub position: Vector2<i32>,
}

#[derive(Debug, Default)]
pub struct ChunkBlocks {
    pub subchunks: Box<[SubChunkBlocks; CHUNK_VOLUME.div_ceil(SUBCHUNK_VOLUME)]>,
}

// The packet relies on this ordering -> leave it like this for performance
/// Ordering: yzx (y being the most significant)
#[derive(Debug)]
pub enum SubChunkBlocks {
    Single(u16),
    Multi(RleVec<u16>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct PaletteEntry {
    name: String,
    _properties: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Clone)]
struct ChunkSectionBlockStates {
    data: Option<LongArray>,
    palette: Vec<PaletteEntry>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct ChunkHeightmaps {
    motion_blocking: LongArray,
    world_surface: LongArray,
}

#[derive(Deserialize, Debug)]
#[expect(dead_code)]
struct ChunkSection {
    #[serde(rename = "Y")]
    y: i32,
    block_states: Option<ChunkSectionBlockStates>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ChunkNbt {
    #[expect(dead_code)]
    data_version: usize,

    #[serde(rename = "sections")]
    sections: Vec<ChunkSection>,

    heightmaps: ChunkHeightmaps,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "Status")]
enum ChunkStatus {
    #[serde(rename = "minecraft:empty")]
    Empty,
    #[serde(rename = "minecraft:structure_starts")]
    StructureStarts,
    #[serde(rename = "minecraft:structure_references")]
    StructureReferences,
    #[serde(rename = "minecraft:biomes")]
    Biomes,
    #[serde(rename = "minecraft:noise")]
    Noise,
    #[serde(rename = "minecraft:surface")]
    Surface,
    #[serde(rename = "minecraft:carvers")]
    Carvers,
    #[serde(rename = "minecraft:features")]
    Features,
    #[serde(rename = "minecraft:initialize_light")]
    InitLight,
    #[serde(rename = "minecraft:light")]
    Light,
    #[serde(rename = "minecraft:spawn")]
    Spawn,
    #[serde(rename = "minecraft:full")]
    Full,
}

/// The Heightmap for a completely empty chunk
impl Default for ChunkHeightmaps {
    fn default() -> Self {
        Self {
            // 0 packed into an i64 7 times.
            motion_blocking: LongArray::new(vec![0; 37]),
            world_surface: LongArray::new(vec![0; 37]),
        }
    }
}

impl ChunkBlocks {
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> u16 {
        self.subchunks[(position.y.get_absolute() / 16) as usize].get_block(
            ChunkRelativeBlockCoordinates {
                x: position.x,
                y: Height::from_absolute(position.y.get_absolute() % 16),
                z: position.z,
            },
        )
    }

    pub fn set_block(&mut self, position: ChunkRelativeBlockCoordinates, block_id: u16) {
        self.subchunks[(position.y.get_absolute() / 16) as usize].set_block(
            ChunkRelativeBlockCoordinates {
                x: position.x,
                y: Height::from_absolute(position.y.get_absolute() % 16),
                z: position.z,
            },
            block_id,
        );
    }

    pub fn set_block_no_heightmap_update(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        block_id: u16,
    ) {
        self.subchunks[(position.y.get_absolute() / 16) as usize].set_block_no_heightmap_update(
            ChunkRelativeBlockCoordinates {
                x: position.x,
                y: Height::from_absolute(position.y.get_absolute() % 16),
                z: position.z,
            },
            block_id,
        );
    }
}

impl Default for SubChunkBlocks {
    fn default() -> Self {
        Self::Single(0)
    }
}

impl SubChunkBlocks {
    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> u16 {
        match self {
            Self::Single(block) => *block,
            Self::Multi(blocks) => blocks[Self::convert_index(position)],
        }
    }

    /// Sets the given block in the chunk
    pub fn set_block(&mut self, position: ChunkRelativeBlockCoordinates, block_id: u16) {
        // TODO @LUK_ESC? update the heightmap
        self.set_block_no_heightmap_update(position, block_id)
    }

    /// Sets the given block in the chunk
    /// Contrary to `set_block` this does not update the heightmap.
    ///
    /// Only use this if you know you don't need to update the heightmap
    /// or if you manually set the heightmap in `empty_with_heightmap`
    pub fn set_block_no_heightmap_update(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        block: u16,
    ) {
        match self {
            Self::Single(single_block) => {
                if *single_block != block {
                    let mut vec = RleVec::new();
                    vec.push_n(SUBCHUNK_VOLUME, *single_block);
                    vec.set(Self::convert_index(position), block);
                    *self = Self::Multi(vec)
                }
            }
            Self::Multi(blocks) => {
                blocks.set(Self::convert_index(position), block);
                if blocks.runs_len() == 1 {
                    *self = Self::Single(block)
                }
            }
        }
    }

    pub fn to_vec(&self) -> Vec<u16> {
        match self {
            Self::Single(block) => {
                vec![*block; SUBCHUNK_VOLUME]
            }
            Self::Multi(blocks) => blocks.to_vec(),
        }
    }

    fn convert_index(index: ChunkRelativeBlockCoordinates) -> usize {
        // % works for negative numbers as intended.
        index.y.get_absolute() as usize * CHUNK_AREA + *index.z as usize * 16 + *index.x as usize
    }
}

impl ChunkData {
    pub fn from_bytes(chunk_data: Vec<u8>, at: Vector2<i32>) -> Result<Self, ChunkParsingError> {
        if fastnbt::from_bytes::<ChunkStatus>(&chunk_data)
            .map_err(|_| ChunkParsingError::FailedReadStatus)?
            != ChunkStatus::Full
        {
            return Err(ChunkParsingError::ChunkNotGenerated);
        }

        let chunk_data = fastnbt::from_bytes::<ChunkNbt>(chunk_data.as_slice())
            .map_err(|e| ChunkParsingError::ErrorDeserializingChunk(e.to_string()))?;

        // this needs to be boxed, otherwise it will cause a stack-overflow
        let mut blocks = ChunkBlocks::default();
        let mut block_index = 0; // which block we're currently at

        for section in chunk_data.sections.into_iter() {
            let block_states = match section.block_states {
                Some(states) => states,
                None => continue, // TODO @lukas0008 this should instead fill all blocks with the only element of the palette
            };

            let palette = block_states
                .palette
                .iter()
                .map(|entry| match BlockState::new(&entry.name) {
                    // Block not found, Often the case when World has an newer or older version then block registry
                    None => BlockState::AIR,
                    Some(state) => state,
                })
                .collect::<Vec<_>>();

            let block_data = match block_states.data {
                None => {
                    // We skipped placing an empty subchunk.
                    // We need to increase the y coordinate of the next subchunk being placed.
                    block_index += SUBCHUNK_VOLUME;
                    continue;
                }
                Some(d) => d,
            }
            .into_inner();

            // How many bits each block has in one of the pallete u64s
            let block_bit_size = {
                let size = 64 - (palette.len() as i64 - 1).leading_zeros();
                max(4, size)
            };
            // How many blocks there are in one of the palletes u64s
            let blocks_in_pallete = 64 / block_bit_size;

            let mask = (1 << block_bit_size) - 1;
            'block_loop: for block in block_data.iter() {
                for i in 0..blocks_in_pallete {
                    let index = (block >> (i * block_bit_size)) & mask;
                    let block = &palette[index as usize];

                    // TODO allow indexing blocks directly so we can just use block_index and save some time?
                    // this is fine because we initalized the heightmap of `blocks`
                    // from the cached value in the world file
                    blocks.set_block_no_heightmap_update(
                        ChunkRelativeBlockCoordinates {
                            z: ((block_index % CHUNK_AREA) / 16).into(),
                            y: Height::from_absolute((block_index / CHUNK_AREA) as u16),
                            x: (block_index % 16).into(),
                        },
                        block.get_id(),
                    );

                    block_index += 1;

                    // if `SUBCHUNK_VOLUME `is not divisible by `blocks_in_pallete` the block_data
                    // can sometimes spill into other subchunks. We avoid that by aborting early
                    if (block_index % SUBCHUNK_VOLUME) == 0 {
                        break 'block_loop;
                    }
                }
            }
        }

        Ok(ChunkData {
            blocks,
            position: at,
            heightmap: chunk_data.heightmaps,
        })
    }
}

#[derive(Error, Debug)]
pub enum ChunkParsingError {
    #[error("Failed reading chunk status")]
    FailedReadStatus,
    #[error("The chunk isn't generated yet")]
    ChunkNotGenerated,
    #[error("Error deserializing chunk: {0}")]
    ErrorDeserializingChunk(String),
}
