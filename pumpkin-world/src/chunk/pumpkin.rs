use std::{
    collections::HashSet,
    fs::OpenOptions,
    hash::RandomState,
    io::{Read, Write},
};

use bitvec::{bits, order, vec::BitVec, view::BitView};
use pumpkin_core::math::ceil_log2;
use serde::de::Error;

use crate::{chunk::ChunkWritingError, level::LevelFolder};

use super::{
    ChunkBlocks, ChunkData, ChunkReader, ChunkReadingError, ChunkSerializingError, ChunkWriter,
    CHUNK_VOLUME, SUBCHUNK_VOLUME,
};

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

        let mut i = 0;

        while i != 24 {
            let palette = {
                let mut palette = Vec::new();

                loop {
                    let block =
                        data.drain(..16)
                            .rev()
                            .fold(0, |acc, bit| if bit { (acc << 1) + 1 } else { acc << 1 });

                    if block == u16::MAX {
                        break;
                    } else {
                        palette.push(block);
                    }
                }
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
                palette
                    .get(
                        b.iter()
                            .rev()
                            .fold(0, |acc, bit| if *bit { (acc << 1) + 1 } else { acc << 1 }),
                    )
                    .unwrap_or(&0)
            }));
            i += 1;
        }

        Ok(ChunkData {
            blocks: ChunkBlocks {
                blocks: blocks
                    .try_into()
                    .or(Err(ChunkReadingError::RegionIsInvalid))?,
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

        for blocks in chunk_data.blocks.blocks.chunks(SUBCHUNK_VOLUME) {
            let mut palette: Vec<&u16> = HashSet::<&u16, RandomState>::from_iter(blocks.iter())
                .into_iter()
                .collect();
            palette.sort();

            let block_bit_size = if palette.len() < 16 {
                4
            } else {
                ceil_log2(palette.len() as u32).max(4)
            } as usize;

            bits.extend_from_bitslice(
                BitVec::<u8, order::Lsb0>::from_vec(
                    palette
                        .iter()
                        .flat_map(|b| b.to_le_bytes())
                        .collect::<Vec<u8>>(),
                )
                .as_bitslice(),
            );
            bits.extend_from_bitslice(bits![1; 16]);

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
