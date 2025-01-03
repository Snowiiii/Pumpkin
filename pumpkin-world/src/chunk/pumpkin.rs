use std::{
    collections::{HashMap, HashSet},
    fs::OpenOptions,
    hash::RandomState,
    io::{Read, Write},
};

use bitvec::{order, vec::BitVec, view::BitView};
use fastnbt::LongArray;
use pumpkin_core::math::ceil_log2;
use rayon::iter::FromParallelIterator;
use serde::de::Error;

use crate::{
    block::block_registry::BLOCK_ID_TO_REGISTRY_ID, chunk::ChunkWritingError, level::LevelFolder,
};

use super::{
    ChunkBlocks, ChunkData, ChunkNbt, ChunkReader, ChunkReadingError, ChunkSection,
    ChunkSectionBlockStates, ChunkSerializingError, ChunkWriter, PaletteEntry, CHUNK_VOLUME,
    SUBCHUNK_VOLUME,
};

// 1.21.4
const WORLD_DATA_VERSION: i32 = 4189;

#[derive(Clone, Default)]
pub struct PumpkinChunkFormat;

impl ChunkReader for PumpkinChunkFormat {
    fn read_chunk(
        &self,
        save_file: &LevelFolder,
        at: &pumpkin_core::math::vector2::Vector2<i32>,
    ) -> Result<super::ChunkData, ChunkReadingError> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(
                save_file
                    .region_folder
                    .join(format!("c.{}.{}.mcp", at.x, at.z)),
            )
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => ChunkReadingError::ChunkNotExist,
                kind => ChunkReadingError::IoError(kind),
            })?;

        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        let mut data: BitVec<u8, order::Lsb0> = BitVec::from_vec(data);

        let mut blocks = Vec::with_capacity(CHUNK_VOLUME);

        while !data.is_empty() {
            let palette = {
                let mut palette = Vec::new();

                let mut block = [0, 0];
                while block != [0xFF, 0xFF] {
                    data.read_exact(&mut block)
                        .map_err(|e| ChunkReadingError::IoError(e.kind()))?;
                    palette.push(u16::from_le_bytes(block));
                }
                palette.pop();
                palette
            };

            let block_bit_size = if palette.len() < 16 {
                4
            } else {
                ceil_log2(palette.len() as u32).max(4)
            } as usize;

            let subchunk_blocks: BitVec<u8, order::Lsb0> =
                data.drain(..SUBCHUNK_VOLUME * block_bit_size).collect();

            blocks.extend(subchunk_blocks.chunks(block_bit_size).map(|b| {
                palette[b
                    .iter()
                    .fold(0, |acc, bit| if *bit { acc << 1 + 1 } else { acc << 1 })]
            }));
        }

        Ok(ChunkData {
            blocks: ChunkBlocks {
                blocks: blocks.try_into().or(Err(ChunkReadingError::RegionIsInvalid))?,
                ..Default::default()
            },
            position: *at,
        })
    }
}

impl ChunkWriter for PumpkinChunkFormat {
    fn write_chunk(
        &self,
        chunk_data: &ChunkData,
        level_folder: &LevelFolder,
        at: &pumpkin_core::math::vector2::Vector2<i32>,
    ) -> Result<(), super::ChunkWritingError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(
                level_folder
                    .region_folder
                    .join(format!("c.{}.{}.mcp", at.x, at.z)),
            )
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        let raw_bytes = self
            .to_bytes(chunk_data)
            .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?;

        file.write_all(&raw_bytes).unwrap();

        Ok(())
    }
}

impl PumpkinChunkFormat {
    pub fn to_bytes(&self, chunk_data: &ChunkData) -> Result<Vec<u8>, ChunkSerializingError> {
        let mut bits: BitVec<u8, order::Lsb0> = BitVec::new();

        for blocks in chunk_data.blocks.blocks.chunks(16 * 16 * 16) {
            let mut palette: Vec<&u16> = HashSet::<&u16, RandomState>::from_iter(blocks.iter())
                .into_iter()
                .collect();
            palette.sort();

            let block_bit_size = if palette.len() < 16 {
                4
            } else {
                ceil_log2(palette.len() as u32).max(4)
            } as usize;

            bits.extend(palette.iter().flat_map(|b| b.to_le_bytes()));
            bits.extend([0xFF, 0xFF]);

            for block in blocks {
                bits.extend_from_bitslice(
                    &palette
                        .binary_search(&block)
                        .map_err(|_| {
                            ChunkSerializingError::ErrorSerializingChunk(
                                fastnbt::error::Error::custom("block not found"),
                            )
                        })?
                        .view_bits::<order::Lsb0>()[..block_bit_size],
                );
            }
        }

        Ok(bits.as_raw_slice().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use pumpkin_core::math::vector2::Vector2;

    use crate::{
        chunk::{anvil::AnvilChunkFormat, ChunkReader, ChunkReadingError},
        level::LevelFolder,
    };

    #[test]
    fn not_existing() {
        let region_path = PathBuf::from("not_existing");
        let result = AnvilChunkFormat.read_chunk(
            &LevelFolder {
                root_folder: PathBuf::from(""),
                region_folder: region_path,
            },
            &Vector2::new(0, 0),
        );
        assert!(matches!(result, Err(ChunkReadingError::ChunkNotExist)));
    }
}
