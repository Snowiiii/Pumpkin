const WORLD_DATA_VERSION: usize = 4082;

use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Read, Seek, Write},
    os::windows::fs::FileExt,
};

use fastnbt::LongArray;
use flate2::{
    bufread::{GzDecoder, GzEncoder, ZlibDecoder, ZlibEncoder},
    Compression as CompressionLevel,
};
use itertools::Itertools;

use crate::{
    block::{BlockId, BlockString},
    chunk::{ChunkSection, ChunkSectionBlockStates, PaletteEntry},
    level::SaveFile,
    WORLD_LOWEST_Y,
};

use super::{
    ChunkData, ChunkNbt, ChunkReader, ChunkReadingError, ChunkSerializingError, ChunkWriter,
    ChunkWritingError, CompressionError,
};

pub struct AnvilChunkFormat;

impl Default for AnvilChunkFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl AnvilChunkFormat {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    /// GZip Compression
    GZip,
    /// ZLib Compression
    ZLib,
    /// Uncompressed (since a version before 1.15.1)
    None,
    /// LZ4 Compression (since 24w04a)
    LZ4,
    /// Custom compression algorithm (since 24w05a)
    Custom,
}

impl Compression {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(Self::GZip),
            2 => Some(Self::ZLib),
            3 => Some(Self::None),
            4 => Some(Self::LZ4),
            // Creative i guess?
            127 => Some(Self::Custom),
            _ => None,
        }
    }

    fn decompress_data(&self, compressed_data: Vec<u8>) -> Result<Vec<u8>, CompressionError> {
        match self {
            Compression::GZip => {
                let mut decoder = GzDecoder::new(&compressed_data[..]);
                let mut chunk_data = Vec::new();
                decoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::GZipError)?;
                Ok(chunk_data)
            }
            Compression::ZLib => {
                let mut decoder = ZlibDecoder::new(&compressed_data[..]);
                let mut chunk_data = Vec::new();
                decoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::ZlibError)?;
                Ok(chunk_data)
            }
            Compression::None => Ok(compressed_data),
            Compression::LZ4 => {
                let mut decoder = lz4::Decoder::new(compressed_data.as_slice())
                    .map_err(CompressionError::LZ4Error)?;
                let mut decompressed_data = Vec::new();
                decoder
                    .read_to_end(&mut decompressed_data)
                    .map_err(CompressionError::LZ4Error)?;
                Ok(decompressed_data)
            }
            Compression::Custom => todo!(),
        }
    }
    fn compress_data(
        &self,
        uncompressed_data: Vec<u8>,
        compression_level: Option<CompressionLevel>,
    ) -> Result<Vec<u8>, CompressionError> {
        let compression_level = compression_level.unwrap_or(CompressionLevel::best());
        match self {
            Compression::GZip => {
                let mut decoder = GzEncoder::new(&uncompressed_data[..], compression_level);
                let mut chunk_data = Vec::new();
                decoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::GZipError)?;
                Ok(chunk_data)
            }
            Compression::ZLib => {
                let mut decoder = ZlibEncoder::new(&uncompressed_data[..], compression_level);
                let mut chunk_data = Vec::new();
                decoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::ZlibError)?;
                Ok(chunk_data)
            }
            Compression::None => Ok(uncompressed_data),
            Compression::LZ4 => {
                let mut compressed_data = Vec::new();
                let mut encoder = lz4::EncoderBuilder::new()
                    .level(compression_level.level())
                    .build(&mut compressed_data)
                    .map_err(CompressionError::LZ4Error)?;
                if let Err(err) = encoder.write_all(&uncompressed_data) {
                    return Err(CompressionError::LZ4Error(err));
                }
                if let (_output, Err(err)) = encoder.finish() {
                    return Err(CompressionError::LZ4Error(err));
                }
                Ok(compressed_data)
            }
            Compression::Custom => todo!(),
        }
    }
}

fn modulus(a: i32, b: i32) -> i32 {
    ((a % b) + b) % b
}

impl ChunkReader for AnvilChunkFormat {
    fn read_chunk(
        &self,
        save_file: &SaveFile,
        at: &pumpkin_core::math::vector2::Vector2<i32>,
    ) -> Result<super::ChunkData, ChunkReadingError> {
        let region = (
            ((at.x as f32) / 32.0).floor() as i32,
            ((at.z as f32) / 32.0).floor() as i32,
        );

        let mut region_file = OpenOptions::new()
            .read(true)
            .open(
                save_file
                    .region_folder
                    .join(format!("r.{}.{}.mca", region.0, region.1)),
            )
            .map_err(|err| match err.kind() {
                std::io::ErrorKind::NotFound => ChunkReadingError::ChunkNotExist,
                kind => ChunkReadingError::IoError(kind),
            })?;

        let mut location_table: [u8; 4096] = [0; 4096];
        let mut timestamp_table: [u8; 4096] = [0; 4096];

        // fill the location and timestamp tables
        region_file
            .read_exact(&mut location_table)
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;
        region_file
            .read_exact(&mut timestamp_table)
            .map_err(|err| ChunkReadingError::IoError(err.kind()))?;

        let chunk_x = modulus(at.x, 32) as u32;
        let chunk_z = modulus(at.z, 32) as u32;
        let table_entry = (chunk_x + chunk_z * 32) * 4;

        let mut offset = vec![0u8];
        offset.extend_from_slice(&location_table[table_entry as usize..table_entry as usize + 3]);
        let offset = u32::from_be_bytes(offset.try_into().unwrap()) as u64 * 4096;
        let size = location_table[table_entry as usize + 3] as usize * 4096;

        if offset == 0 && size == 0 {
            return Err(ChunkReadingError::ChunkNotExist);
        }

        // Read the file using the offset and size
        let mut file_buf = {
            region_file
                .seek(std::io::SeekFrom::Start(offset))
                .map_err(|_| ChunkReadingError::RegionIsInvalid)?;
            let mut out = vec![0; size];
            region_file
                .read_exact(&mut out)
                .map_err(|_| ChunkReadingError::RegionIsInvalid)?;
            out
        };

        // TODO: check checksum to make sure chunk is not corrupted
        let header = file_buf.drain(0..5).collect_vec();

        let compression = Compression::from_byte(header[4])
            .ok_or_else(|| ChunkReadingError::Compression(CompressionError::UnknownCompression))?;

        let size = u32::from_be_bytes(header[..4].try_into().unwrap());

        // size includes the compression scheme byte, so we need to subtract 1
        let chunk_data = file_buf.drain(0..size as usize - 1).collect_vec();
        let decompressed_chunk = compression
            .decompress_data(chunk_data)
            .map_err(ChunkReadingError::Compression)?;

        ChunkData::from_bytes(decompressed_chunk, *at).map_err(ChunkReadingError::ParsingError)
    }
}

impl ChunkWriter for AnvilChunkFormat {
    fn write_chunk(
        &self,
        chunk_data: &ChunkData,
        save_file: &SaveFile,
        at: pumpkin_core::math::vector2::Vector2<i32>,
    ) -> Result<(), super::ChunkWritingError> {
        // TODO: update timestamp

        let bytes = self
            .chunk_to_bytes(chunk_data)
            .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?;
        let bytes = Compression::ZLib
            .compress_data(bytes, Some(CompressionLevel::best()))
            .map_err(|e| ChunkWritingError::Compression(e))?;

        let region = (
            ((at.x as f32) / 32.0).floor() as i32,
            ((at.z as f32) / 32.0).floor() as i32,
        );

        let mut region_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(
                save_file
                    .region_folder
                    .join(format!("r.{}.{}.mca", region.0, region.1)),
            )
            .map_err(|err| match err.kind() {
                kind => ChunkWritingError::IoError(kind),
            })?;

        let mut location_table: [u8; 4096] = [0; 4096];
        let mut timestamp_table: [u8; 4096] = [0; 4096];

        let file_meta = region_file
            .metadata()
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

        // fill the location and timestamp tables if they exist
        if file_meta.len() >= 4096 * 2 {
            region_file
                .read_exact(&mut location_table)
                .map_err(|err| ChunkWritingError::IoError(err.kind()))?;
            region_file
                .read_exact(&mut timestamp_table)
                .map_err(|err| ChunkWritingError::IoError(err.kind()))?;
        }

        // write new location and timestamp table
        if let Err(err) = region_file.seek_write(&[location_table, timestamp_table].concat(), 0) {
            return Err(ChunkWritingError::IoError(err.kind()));
        }

        let chunk_x = modulus(at.x, 32) as u32;
        let chunk_z = modulus(at.z, 32) as u32;

        let table_entry = (chunk_x + chunk_z * 32) * 4;

        let mut offset = vec![0u8];
        offset.extend_from_slice(&location_table[table_entry as usize..table_entry as usize + 3]);
        let at_offset = u32::from_be_bytes(offset.try_into().unwrap()) as u64 * 4096;
        let at_size = location_table[table_entry as usize + 3] as usize * 4096;

        let mut end_index = 4096 * 2;
        if at_offset != 0 || at_size != 0 {
            // move other chunks earlier, if there is a hole
            for (other_offset, other_size, other_table_entry) in location_table
                .chunks(4)
                .enumerate()
                .filter_map(|(index, v)| {
                    if table_entry == index as u32 {
                        return None;
                    }
                    let mut offset = vec![0u8];
                    offset.extend_from_slice(&v[0..3]);
                    let offset = u32::from_be_bytes(offset.try_into().unwrap()) as u64 * 4096;
                    let size = v[3] as usize * 4096;
                    if offset == 0 && size == 0 {
                        return None;
                    }
                    Some((offset, size, index * 4))
                })
                .sorted_by(|a, b| a.cmp(b))
            {
                if at_offset > other_offset {
                    continue;
                }
                let mut buf = vec![0u8; other_size];
                region_file
                    .seek_read(&mut buf, other_offset)
                    .expect(&format!(
                        "Region file r.-{},{}.mca got corrupted, sorry",
                        region.0, region.1,
                    ));

                region_file
                    .seek_write(&buf, other_offset - at_size as u64)
                    .expect(&format!(
                        "Region file r.-{},{}.mca got corrupted, sorry",
                        region.0, region.1,
                    ));
                let location_bytes =
                    &(((other_offset - at_size as u64) / 4096) as u32).to_be_bytes()[0..3];
                let size_bytes = [(other_size / 4096) as u8];
                let location_entry = [location_bytes, &size_bytes].concat();
                location_table[other_table_entry..other_table_entry + 4]
                    .as_mut()
                    .copy_from_slice(&location_entry);

                end_index = other_offset + other_size as u64;
            }
        } else {
            for (offset, size) in location_table.chunks(4).filter_map(|v| {
                let mut offset = vec![0u8];
                offset.extend_from_slice(&v[0..3]);
                let offset = u32::from_be_bytes(offset.try_into().unwrap()) as u64 * 4096;
                let size = v[3] as usize * 4096;
                if offset == 0 && size == 0 {
                    return None;
                }
                Some((offset, size))
            }) {
                end_index = u64::max(offset + size as u64, end_index);
            }
        }

        region_file.seek_write(&bytes, end_index).expect(&format!(
            "Region file r.-{},{}.mca got corrupted, sorry",
            region.0, region.1,
        ));

        Ok(())
    }
}

impl AnvilChunkFormat {
    pub fn chunk_to_bytes(&self, chunk_data: &ChunkData) -> Result<Vec<u8>, ChunkSerializingError> {
        let mut sections = Vec::new();

        for (i, blocks) in chunk_data.blocks.blocks.chunks(16 * 16 * 16).enumerate() {
            // get unique blocks
            let unique_blocks = blocks.iter().dedup().collect_vec();
            let palette = HashMap::<BlockId, (&'static BlockString, usize)>::from_iter(
                unique_blocks.iter().enumerate().map(|(i, v)| {
                    (
                        **v,
                        (
                            v.get_block_string()
                                .expect("Tried saving a block which does not exist."),
                            i,
                        ),
                    )
                }),
            );

            let block_bit_size = {
                let size = 64 - (palette.len() as i64 - 1).leading_zeros();
                std::cmp::max(4, size)
            } as usize;
            let blocks_in_pack = (64 / block_bit_size) as usize;
            let mut section_longs = Vec::new();

            let mut current_pack_long = 0i64;

            for block_pack in blocks.chunks(blocks_in_pack) {
                for block in block_pack {
                    let index = palette.get(block).expect("Just added all unique").1;
                    current_pack_long = current_pack_long << block_bit_size | index as i64;
                }
                section_longs.push(current_pack_long);
                current_pack_long = 0;
            }

            sections.push(ChunkSection {
                y: i as i32 * 16 - WORLD_LOWEST_Y as i32,
                block_states: Some(ChunkSectionBlockStates {
                    data: Some(LongArray::new(section_longs)),
                    palette: palette
                        .into_iter()
                        .map(|entry| PaletteEntry {
                            name: entry.1 .0.name.to_string(),
                            properties: if entry.1 .0.properties.is_empty() {
                                None
                            } else {
                                Some(entry.1 .0.properties.clone())
                            },
                        })
                        .collect(),
                }),
            });
        }

        let nbt = ChunkNbt {
            data_version: WORLD_DATA_VERSION,
            heightmaps: chunk_data.blocks.heightmap.clone(),
            sections,
        };

        let bytes = fastnbt::to_bytes(&nbt);

        bytes.map_err(|err| ChunkSerializingError::ErrorSerializingChunk(err))
    }
}
