use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
};

use bytes::*;
use fastnbt::LongArray;
use flate2::bufread::{GzDecoder, GzEncoder, ZlibDecoder, ZlibEncoder};

use crate::{
    block::block_registry::BLOCK_ID_TO_REGISTRY_ID, chunk::ChunkWritingError, level::LevelFolder,
};

use super::{
    ChunkData, ChunkNbt, ChunkReader, ChunkReadingError, ChunkSection, ChunkSectionBlockStates,
    ChunkSerializingError, ChunkWriter, CompressionError, PaletteEntry,
};

// 1.21.4
const WORLD_DATA_VERSION: i32 = 4189;

#[derive(Clone, Default)]
pub struct AnvilChunkFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Compression {
    /// GZip Compression
    GZip = 1,
    /// ZLib Compression
    ZLib = 2,
    /// LZ4 Compression (since 24w04a)
    LZ4 = 4,
    /// Custom compression algorithm (since 24w05a)
    Custom = 127,
}

impl Compression {
    /// Returns Ok when a compression is found otherwise an Err
    #[allow(clippy::result_unit_err)]
    pub fn from_byte(byte: u8) -> Result<Option<Self>, ()> {
        match byte {
            1 => Ok(Some(Self::GZip)),
            2 => Ok(Some(Self::ZLib)),
            // Uncompressed (since a version before 1.15.1)
            3 => Ok(None),
            4 => Ok(Some(Self::LZ4)),
            127 => Ok(Some(Self::Custom)),
            // Unknown format
            _ => Err(()),
        }
    }

    fn decompress_data(&self, compressed_data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        match self {
            Compression::GZip => {
                let mut decoder = GzDecoder::new(compressed_data);
                let mut chunk_data = Vec::new();
                decoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::GZipError)?;
                Ok(chunk_data)
            }
            Compression::ZLib => {
                let mut decoder = ZlibDecoder::new(compressed_data);
                let mut chunk_data = Vec::new();
                decoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::ZlibError)?;
                Ok(chunk_data)
            }
            Compression::LZ4 => {
                let mut decoder =
                    lz4::Decoder::new(compressed_data).map_err(CompressionError::LZ4Error)?;
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
        uncompressed_data: &[u8],
        compression_level: u32,
    ) -> Result<Vec<u8>, CompressionError> {
        match self {
            Compression::GZip => {
                let mut decoder = GzEncoder::new(
                    uncompressed_data,
                    flate2::Compression::new(compression_level),
                );
                let mut chunk_data = Vec::new();
                decoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::GZipError)?;
                Ok(chunk_data)
            }
            Compression::ZLib => {
                let mut decoder = ZlibEncoder::new(
                    uncompressed_data,
                    flate2::Compression::new(compression_level),
                );
                let mut chunk_data = Vec::new();
                decoder
                    .read_to_end(&mut chunk_data)
                    .map_err(CompressionError::ZlibError)?;
                Ok(chunk_data)
            }
            Compression::LZ4 => {
                let mut compressed_data = Vec::new();
                let mut encoder = lz4::EncoderBuilder::new()
                    .level(compression_level)
                    .build(&mut compressed_data)
                    .map_err(CompressionError::LZ4Error)?;
                if let Err(err) = encoder.write_all(uncompressed_data) {
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

impl ChunkReader for AnvilChunkFormat {
    fn read_chunk(
        &self,
        save_file: &LevelFolder,
        at: &pumpkin_core::math::vector2::Vector2<i32>,
    ) -> Result<super::ChunkData, ChunkReadingError> {
        let region = (at.x >> 5, at.z >> 5);

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

        let chunk_x = at.x & 0x1F;
        let chunk_z = at.z & 0x1F;
        let table_entry = (chunk_x + chunk_z * 32) * 4;

        let mut offset = BytesMut::new();
        offset.put_u8(0);
        offset.extend_from_slice(&location_table[table_entry as usize..table_entry as usize + 3]);
        let offset = offset.get_u32() as u64 * 4096;
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

        let mut header: Bytes = file_buf.drain(0..5).collect();
        if header.remaining() != 5 {
            return Err(ChunkReadingError::InvalidHeader);
        }
        let size = header.get_u32();
        let compression = header.get_u8();

        let compression = Compression::from_byte(compression)
            .map_err(|_| ChunkReadingError::Compression(CompressionError::UnknownCompression))?;

        // size includes the compression scheme byte, so we need to subtract 1
        let chunk_data: Vec<u8> = file_buf.drain(0..size as usize - 1).collect();

        let decompressed_chunk = if let Some(compression) = compression {
            compression
                .decompress_data(&chunk_data)
                .map_err(ChunkReadingError::Compression)?
        } else {
            chunk_data
        };

        ChunkData::from_bytes(&decompressed_chunk, *at).map_err(ChunkReadingError::ParsingError)
    }
}

impl ChunkWriter for AnvilChunkFormat {
    fn write_chunk(
        &self,
        chunk_data: &ChunkData,
        level_folder: &LevelFolder,
        at: &pumpkin_core::math::vector2::Vector2<i32>,
    ) -> Result<(), super::ChunkWritingError> {
        // TODO: update timestamp

        let bytes = self
            .to_bytes(chunk_data)
            .map_err(|err| ChunkWritingError::ChunkSerializingError(err.to_string()))?;
        // TODO: config
        let compression = Compression::ZLib;
        let bytes = compression
            // TODO: config
            .compress_data(&bytes, 6)
            .map_err(ChunkWritingError::Compression)?;

        let region = (at.x >> 5, at.z >> 5);

        let mut region_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(
                level_folder
                    .region_folder
                    .join(format!("./r.{}.{}.mca", region.0, region.1)),
            )
            .map_err(|err| ChunkWritingError::IoError(err.kind()))?;

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

        let chunk_x = at.x & 0x1F;
        let chunk_z = at.z & 0x1F;

        let table_entry = (chunk_x + chunk_z * 32) * 4;

        let mut offset = BytesMut::new();
        offset.put_u8(0);
        offset.extend_from_slice(&location_table[table_entry as usize..table_entry as usize + 3]);
        let at_offset = offset.get_u32() as u64 * 4096;
        let at_size = location_table[table_entry as usize + 3] as usize * 4096;

        let mut end_index = 4096 * 2;
        if at_offset != 0 || at_size != 0 {
            // move other chunks earlier, if there is a hole
            for (other_offset, other_size, other_table_entry) in location_table
                .chunks(4)
                .enumerate()
                .filter_map(|(index, v)| {
                    if table_entry / 4 == index as i32 {
                        return None;
                    }
                    let mut offset = BytesMut::new();
                    offset.put_u8(0);
                    offset.extend_from_slice(&v[0..3]);
                    let offset = offset.get_u32() as u64 * 4096;
                    let size = v[3] as usize * 4096;
                    if offset == 0 && size == 0 {
                        return None;
                    }
                    Some((offset, size, index * 4))
                })
                .collect::<Vec<_>>()
            {
                if at_offset > other_offset {
                    continue;
                }

                fn read_at_most(file: &mut std::fs::File, size: usize) -> std::io::Result<Vec<u8>> {
                    let mut buf = vec![0u8; size];

                    let mut cursor = 0;
                    loop {
                        match file.read(&mut buf[cursor..])? {
                            0 => break,
                            bytes_read => {
                                cursor += bytes_read;
                            }
                        }
                    }

                    Ok(buf)
                }

                region_file.seek(SeekFrom::Start(other_offset)).unwrap(); // TODO
                let buf = match read_at_most(&mut region_file, other_size) {
                    Ok(v) => v,
                    Err(_) => panic!(
                        "Region file r.-{},{}.mca got corrupted, sorry",
                        region.0, region.1
                    ),
                };

                region_file
                    .seek(SeekFrom::Start(other_offset - at_size as u64))
                    .unwrap(); // TODO
                region_file.write_all(&buf).unwrap_or_else(|_| {
                    panic!(
                        "Region file r.-{},{}.mca got corrupted, sorry",
                        region.0, region.1
                    )
                });
                let location_bytes =
                    &(((other_offset - at_size as u64) / 4096) as u32).to_be_bytes()[1..4];
                let size_bytes = [(other_size.div_ceil(4096)) as u8];
                let location_entry = [location_bytes, &size_bytes].concat();
                location_table[other_table_entry..other_table_entry + 4]
                    .as_mut()
                    .copy_from_slice(&location_entry);

                end_index = (other_offset as isize + other_size as isize - at_size as isize) as u64;
            }
        } else {
            for (offset, size) in location_table.chunks(4).filter_map(|v| {
                let mut offset = BytesMut::new();
                offset.put_u8(0);
                offset.extend_from_slice(&v[0..3]);
                let offset = offset.get_u32() as u64 * 4096;
                let size = v[3] as usize * 4096;
                if offset == 0 && size == 0 {
                    return None;
                }
                Some((offset, size))
            }) {
                end_index = u64::max(offset + size as u64, end_index);
            }
        }

        let location_bytes = &(end_index as u32 / 4096).to_be_bytes()[1..4];
        let size_bytes = [(bytes.len().div_ceil(4096)) as u8];
        location_table[table_entry as usize..table_entry as usize + 4]
            .as_mut()
            .copy_from_slice(&[location_bytes, &size_bytes].concat());

        // write new location and timestamp table

        region_file.seek(SeekFrom::Start(0)).unwrap(); // TODO
        if let Err(err) = region_file.write_all(&[location_table, timestamp_table].concat()) {
            return Err(ChunkWritingError::IoError(err.kind()));
        }
        // dbg!(&location_table.iter().map(|v| v.to_string()).join(","));

        region_file.seek(SeekFrom::Start(end_index)).unwrap(); // TODO

        let mut header: BytesMut = BytesMut::with_capacity(5);
        // total chunk size includes the byte representing the compression
        // scheme, so +1.
        header.put_u32(bytes.len() as u32 + 1);
        // compression scheme
        header.put_u8(compression as u8);
        region_file
            .write_all(&header)
            .expect("Failed to write header");
        region_file.write_all(&bytes).unwrap_or_else(|_| {
            panic!(
                "Region file r.-{},{}.mca got corrupted, sorry",
                region.0, region.1
            )
        });

        region_file
            .write_all(&vec![0u8; 4096])
            .expect("Failed to add padding");

        Ok(())
    }
}

impl AnvilChunkFormat {
    pub fn to_bytes(&self, chunk_data: &ChunkData) -> Result<Vec<u8>, ChunkSerializingError> {
        let mut sections = Vec::new();

        for (i, blocks) in chunk_data.blocks.blocks.chunks(16 * 16 * 16).enumerate() {
            // get unique blocks
            let palette = HashMap::<u16, (&String, usize)>::from_iter(
                blocks.iter().enumerate().map(|(i, v)| {
                    (
                        *v,
                        (
                            BLOCK_ID_TO_REGISTRY_ID
                                .get(v)
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
            let blocks_in_pack = 64 / block_bit_size;
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
                y: i as i8,
                block_states: Some(ChunkSectionBlockStates {
                    data: Some(LongArray::new(section_longs)),
                    palette: palette
                        .into_iter()
                        .map(|entry| PaletteEntry {
                            name: entry.1 .0.clone(),
                            _properties: None,
                        })
                        .collect(),
                }),
            });
        }

        let nbt = ChunkNbt {
            data_version: WORLD_DATA_VERSION,
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
