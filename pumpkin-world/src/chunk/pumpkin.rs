use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Read, Write},
};

use fastnbt::LongArray;
use pumpkin_core::math::ceil_log2;

use crate::{
    block::block_registry::BLOCK_ID_TO_REGISTRY_ID, chunk::ChunkWritingError, level::LevelFolder,
};

use super::{
    ChunkData, ChunkNbt, ChunkReader, ChunkReadingError, ChunkSection, ChunkSectionBlockStates,
    ChunkSerializingError, ChunkWriter, PaletteEntry,
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

        ChunkData::from_bytes(&data, *at).map_err(ChunkReadingError::ParsingError)
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
        let mut sections = Vec::new();

        for (i, blocks) in chunk_data.blocks.blocks.chunks(16 * 16 * 16).enumerate() {
            // get unique blocks
            let palette = HashMap::<u16, &String>::from_iter(blocks.iter().map(|v| {
                (
                    *v,
                    BLOCK_ID_TO_REGISTRY_ID
                        .get(v)
                        .expect("Tried saving a block which does not exist."),
                )
            }));
            let palette = HashMap::<u16, (&String, usize)>::from_iter(
                palette
                    .into_iter()
                    .enumerate()
                    .map(|(index, (block_id, registry_str))| (block_id, (registry_str, index))),
            );

            let block_bit_size = if palette.len() < 16 {
                4
            } else {
                ceil_log2(palette.len() as u32).max(4)
            };
            let _blocks_in_pack = 64 / block_bit_size;

            let mut section_longs = Vec::new();
            let mut current_pack_long: i64 = 0;
            let mut bits_used_in_pack: u32 = 0;

            for block in blocks {
                let index = palette.get(block).expect("Just added all unique").1;
                current_pack_long |= (index as i64) << bits_used_in_pack;
                bits_used_in_pack += block_bit_size as u32;

                if bits_used_in_pack >= 64 {
                    section_longs.push(current_pack_long);
                    current_pack_long = 0;
                    bits_used_in_pack = 0;
                }
            }

            if bits_used_in_pack > 0 {
                section_longs.push(current_pack_long);
            }

            sections.push(ChunkSection {
                y: i as i8,
                block_states: Some(ChunkSectionBlockStates {
                    data: Some(LongArray::new(section_longs)),
                    palette: palette
                        .into_iter()
                        .map(|entry| PaletteEntry {
                            name: entry.1 .0.clone(),
                            properties: None,
                        })
                        .collect(),
                }),
            });
        }

        let nbt = ChunkNbt {
            data_version: WORLD_DATA_VERSION,
            x_pos: chunk_data.position.x,
            z_pos: chunk_data.position.z,
            status: super::ChunkStatus::Full,
            heightmaps: chunk_data.blocks.heightmap.clone(),
            sections,
        };

        let bytes = fastnbt::to_bytes(&nbt);

        bytes.map_err(ChunkSerializingError::ErrorSerializingChunk)
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
