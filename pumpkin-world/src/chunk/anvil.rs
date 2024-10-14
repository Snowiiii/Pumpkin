use std::{
    fs::OpenOptions,
    io::{Read, Seek},
};

use flate2::bufread::{GzDecoder, ZlibDecoder};
use itertools::Itertools;

use crate::level::SaveFile;

use super::{ChunkData, ChunkReader, ChunkReadingError, CompressionError};

pub struct AnvilChunkReader {}

impl Default for AnvilChunkReader {
    fn default() -> Self {
        Self::new()
    }
}

impl AnvilChunkReader {
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
}

impl ChunkReader for AnvilChunkReader {
    fn read_chunk(
        &self,
        save_file: &SaveFile,
        at: pumpkin_core::math::vector2::Vector2<i32>,
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

        let modulus = |a: i32, b: i32| ((a % b) + b) % b;
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

        ChunkData::from_bytes(decompressed_chunk, at).map_err(ChunkReadingError::ParsingError)
    }
}
