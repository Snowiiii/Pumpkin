use crate::level::SaveFile;
use byteorder::{BigEndian, ReadBytesExt};
use flate2::bufread::{GzDecoder, ZlibDecoder};
use pumpkin_core::math::vector2::Vector2;
use std::fs::File;
use std::io::{Read, Seek};

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
    fn read_chunk(&self, save_file: &SaveFile, at: Vector2<i32>) -> Result<ChunkData, ChunkReadingError> {
        let region = (
            ((at.x as f32) / 32.0).floor() as i32,
            ((at.z as f32) / 32.0).floor() as i32,
        );

        let mut region_file =
            File::open(
                save_file
                    .region_folder
                    .join(format!("r.{}.{}.mca", region.0, region.1)),
            )
                .map_err(|err| match err.kind() {
                    std::io::ErrorKind::NotFound => ChunkReadingError::ChunkNotExist,
                    kind => ChunkReadingError::IoError(kind),
                })?;


        let index = 4 * ((at.x as u64 % 32) + (at.z as u64 % 32) * 32);
        region_file.seek(std::io::SeekFrom::Start(index * 4)).map_err(|_| ChunkReadingError::RegionIsInvalid)?;
        let location = region_file.read_u32::<BigEndian>().map_err(|_| ChunkReadingError::RegionIsInvalid)?;

        let offset = ((location >> 8) & 0xFFFFFF) * 4096;
        let size = (location & 0xFF) * 4096;

        if offset == 0 && size == 0 {
            return Err(ChunkReadingError::ChunkNotExist);
        }

        let mut chunk_data = vec![0u8; size as usize];
        region_file.seek(std::io::SeekFrom::Start(offset as u64)).map_err(|_| ChunkReadingError::RegionIsInvalid)?;
        region_file.read_exact(&mut chunk_data).map_err(|_| ChunkReadingError::RegionIsInvalid)?;

        let chunk_header = chunk_data[0..4].to_vec();
        let chunk_compressed_data = chunk_data[5..].to_vec();
        let uncompressed_size = u32::from(chunk_header[0]) << 24
            | u32::from(chunk_header[1]) << 16
            | u32::from(chunk_header[2]) << 8
            | u32::from(chunk_header[3]);
        let compression_type = chunk_data[4];
        let res = match compression_type {
            1 => {
                let mut decompressed_data = Vec::new();
                let mut decoder = flate2::read::GzDecoder::new(&chunk_compressed_data[..]);
                decoder.read_to_end(&mut decompressed_data).unwrap();
                Some(decompressed_data)
            }
            2 => {
                let mut decompressor = zune_inflate::DeflateDecoder::new(&chunk_compressed_data[..]);
                Some(decompressor.decode_zlib().unwrap())
            }
            3 => Some(chunk_compressed_data),
            4 => {
                let mut decompressed_data = vec![0; uncompressed_size as usize];
                lz4::Decoder::new(&chunk_compressed_data[..])
                    .and_then(|mut decoder| decoder.read_exact(&mut decompressed_data))
                    .map_err(|e| ChunkReadingError::Compression(CompressionError::LZ4Error(e)))?;
                Some(decompressed_data)
            }
            _ => {
                None
            }
        }.ok_or(ChunkReadingError::Compression(CompressionError::UnknownCompression))?;
        ChunkData::from_bytes(res, at).map_err(ChunkReadingError::ParsingError)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_bad_load_fails() {
        let file_path = PathBuf::from("../.etc/shouldnotexist/");
        let result = AnvilChunkReader::new().read_chunk(&SaveFile {
            root_folder: PathBuf::from(""),
            region_folder: file_path.clone(),
        }, Vector2::new(0, 0));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_chunk() {
        let file_path = PathBuf::from("../.etc/regions");
        let loaded_file = AnvilChunkReader::new().read_chunk(&SaveFile {
            root_folder: PathBuf::from(""),
            region_folder: file_path.clone(),
        }, Vector2::new(0, 0));
        assert!(loaded_file.is_ok());
        assert_eq!(loaded_file.unwrap().position, Vector2::new(0, 0));
    }
}