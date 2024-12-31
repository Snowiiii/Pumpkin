use fastnbt::LongArray;
use pumpkin_core::math::{ceil_log2, vector2::Vector2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Index;
use thiserror::Error;

use crate::{
    block::BlockState,
    coordinates::{ChunkRelativeBlockCoordinates, Height},
    level::LevelFolder,
    WORLD_HEIGHT,
};

pub mod anvil;

const CHUNK_AREA: usize = 16 * 16;
const SUBCHUNK_VOLUME: usize = CHUNK_AREA * 16;
const CHUNK_VOLUME: usize = CHUNK_AREA * WORLD_HEIGHT;

pub trait ChunkReader: Sync + Send {
    fn read_chunk(
        &self,
        save_file: &LevelFolder,
        at: &Vector2<i32>,
    ) -> Result<ChunkData, ChunkReadingError>;
}

pub trait ChunkWriter: Send + Sync {
    fn write_chunk(
        &self,
        chunk: &ChunkData,
        level_folder: &LevelFolder,
        at: &Vector2<i32>,
    ) -> Result<(), ChunkWritingError>;
}

#[derive(Error, Debug)]
pub enum ChunkReadingError {
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Invalid header")]
    InvalidHeader,
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
pub enum ChunkWritingError {
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Compression error {0}")]
    Compression(CompressionError),
    #[error("Chunk serializing error: {0}")]
    ChunkSerializingError(String),
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

pub struct ChunkData {
    pub blocks: ChunkBlocks,
    pub position: Vector2<i32>,
}
pub struct ChunkBlocks {
    // TODO make this a Vec that doesn't store the upper layers that only contain air

    // The packet relies on this ordering -> leave it like this for performance
    /// Ordering: yzx (y being the most significant)
    blocks: Box<[u16; CHUNK_VOLUME]>,

    /// See `https://minecraft.wiki/w/Heightmap` for more info
    pub heightmap: ChunkHeightmaps,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct PaletteEntry {
    // block name
    name: String,
    properties: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub struct ChunkHeightmaps {
    // #[serde(with = "LongArray")]
    motion_blocking: LongArray,
    // #[serde(with = "LongArray")]
    world_surface: LongArray,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChunkSection {
    #[serde(rename = "Y")]
    y: i8,
    block_states: Option<ChunkSectionBlockStates>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChunkSectionBlockStates {
    //  #[serde(with = "LongArray")]
    data: Option<LongArray>,
    palette: Vec<PaletteEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ChunkNbt {
    data_version: i32,
    #[serde(rename = "xPos")]
    x_pos: i32,
    // #[serde(rename = "yPos")]
    //y_pos: i32,
    #[serde(rename = "zPos")]
    z_pos: i32,
    status: ChunkStatus,
    #[serde(rename = "sections")]
    sections: Vec<ChunkSection>,
    heightmaps: ChunkHeightmaps,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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

impl Default for ChunkBlocks {
    fn default() -> Self {
        Self {
            blocks: Box::new([0; CHUNK_VOLUME]),
            heightmap: ChunkHeightmaps::default(),
        }
    }
}

impl ChunkBlocks {
    pub const fn len(&self) -> usize {
        self.blocks.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub const fn subchunks_len(&self) -> usize {
        self.blocks.len().div_ceil(SUBCHUNK_VOLUME)
    }

    pub fn empty_with_heightmap(heightmap: ChunkHeightmaps) -> Self {
        Self {
            blocks: Box::new([0; CHUNK_VOLUME]),
            heightmap,
        }
    }

    /// Gets the given block in the chunk
    pub fn get_block(&self, position: ChunkRelativeBlockCoordinates) -> Option<u16> {
        self.blocks.get(Self::convert_index(position)).copied()
    }

    /// Sets the given block in the chunk, returning the old block
    pub fn set_block(&mut self, position: ChunkRelativeBlockCoordinates, block_id: u16) -> u16 {
        // TODO @LUK_ESC? update the heightmap
        self.set_block_no_heightmap_update(position, block_id)
    }

    /// Sets the given block in the chunk, returning the old block
    /// Contrary to `set_block` this does not update the heightmap.
    ///
    /// Only use this if you know you don't need to update the heightmap
    /// or if you manually set the heightmap in `empty_with_heightmap`
    pub fn set_block_no_heightmap_update(
        &mut self,
        position: ChunkRelativeBlockCoordinates,
        block: u16,
    ) -> u16 {
        std::mem::replace(&mut self.blocks[Self::convert_index(position)], block)
    }

    pub fn iter_subchunks(&self) -> impl Iterator<Item = &[u16; SUBCHUNK_VOLUME]> {
        self.blocks
            .chunks(SUBCHUNK_VOLUME)
            .map(|subchunk| subchunk.try_into().unwrap())
    }

    fn convert_index(index: ChunkRelativeBlockCoordinates) -> usize {
        // % works for negative numbers as intended.
        index.y.get_absolute() as usize * CHUNK_AREA + *index.z as usize * 16 + *index.x as usize
    }

    #[expect(dead_code)]
    fn calculate_heightmap(&self) -> ChunkHeightmaps {
        // figure out how LongArray is formatted
        // figure out how to find out if block is motion blocking
        todo!()
    }
}

impl Index<ChunkRelativeBlockCoordinates> for ChunkBlocks {
    type Output = u16;

    fn index(&self, index: ChunkRelativeBlockCoordinates) -> &Self::Output {
        &self.blocks[Self::convert_index(index)]
    }
}

// I can't use an tag because it will break ChunkNBT, but status need to have a big S, so "Status"
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ChunkStatusWrapper {
    status: ChunkStatus,
}

impl ChunkData {
    pub fn from_bytes(
        chunk_data: &[u8],
        position: Vector2<i32>,
    ) -> Result<Self, ChunkParsingError> {
        if fastnbt::from_bytes::<ChunkStatusWrapper>(chunk_data)
            .map_err(|_| ChunkParsingError::FailedReadStatus)?
            .status
            != ChunkStatus::Full
        {
            return Err(ChunkParsingError::ChunkNotGenerated);
        }

        let chunk_data = fastnbt::from_bytes::<ChunkNbt>(chunk_data)
            .map_err(|e| ChunkParsingError::ErrorDeserializingChunk(e.to_string()))?;

        if chunk_data.x_pos != position.x || chunk_data.z_pos != position.z {
            log::error!(
                "Expected chunk at {}:{}, but got {}:{}",
                position.x,
                position.z,
                chunk_data.x_pos,
                chunk_data.z_pos
            );
            // lets still continue
        }

        // this needs to be boxed, otherwise it will cause a stack-overflow
        let mut blocks = ChunkBlocks::empty_with_heightmap(chunk_data.heightmaps);
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
            };

            // How many bits each block has in one of the palette u64s
            let block_bit_size = if palette.len() < 16 {
                4
            } else {
                ceil_log2(palette.len() as u32).max(4)
            };
            // How many blocks there are in one of the palettes u64s
            let blocks_in_palette = 64 / block_bit_size;

            let mask = (1 << block_bit_size) - 1;
            'block_loop: for block in block_data.iter() {
                for i in 0..blocks_in_palette {
                    let index = (block >> (i * block_bit_size)) & mask;
                    let block = &palette[index as usize];

                    // TODO allow indexing blocks directly so we can just use block_index and save some time?
                    // this is fine because we initialized the heightmap of `blocks`
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

                    // if `SUBCHUNK_VOLUME `is not divisible by `blocks_in_palette` the block_data
                    // can sometimes spill into other subchunks. We avoid that by aborting early
                    if (block_index % SUBCHUNK_VOLUME) == 0 {
                        break 'block_loop;
                    }
                }
            }
        }

        Ok(ChunkData { blocks, position })
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

#[derive(Error, Debug)]
pub enum ChunkSerializingError {
    #[error("Error serializing chunk: {0}")]
    ErrorSerializingChunk(fastnbt::error::Error),
}
