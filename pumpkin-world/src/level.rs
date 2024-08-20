use std::{
    fs::OpenOptions,
    io::{Read, Seek},
    path::PathBuf,
};

use flate2::{bufread::ZlibDecoder, read::GzDecoder};
use itertools::Itertools;
use rayon::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;

use crate::chunk::ChunkData;

#[allow(dead_code)]
/// The Level represents a
pub struct Level {
    root_folder: PathBuf,
    region_folder: PathBuf,
}

#[derive(Error, Debug)]
pub enum WorldError {
    // using ErrorKind instead of Error, beacuse the function read_chunks and read_region_chunks is designed to return an error on a per-chunk basis, while std::io::Error does not implement Copy or Clone
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Region not found")]
    RegionNotFound,
    #[error("Region is invalid")]
    RegionIsInvalid,
    #[error("Chunk not found")]
    ChunkNotFound,
    #[error("Compression Error")]
    Compression(CompressionError),
    #[error("Error deserializing chunk: {0}")]
    ErrorDeserializingChunk(String),
    #[error("The requested block state id does not exist")]
    BlockStateIdNotFound,
}

#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("Compression scheme not recognised")]
    UnknownCompression,
    #[error("Error while working with zlib compression: {0}")]
    ZlibError(std::io::Error),
    #[error("Error while working with Gzip compression: {0}")]
    GZipError(std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    Gzip,
    Zlib,
    None,
    LZ4,
}

impl Compression {
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(Self::Gzip),
            2 => Some(Self::Zlib),
            3 => Some(Self::None),
            4 => Some(Self::LZ4),
            _ => None,
        }
    }
}

impl Level {
    pub fn from_root_folder(root_folder: PathBuf) -> Self {
        assert!(root_folder.exists(), "World root folder does not exist!");
        let region_folder = root_folder.join("region");
        assert!(
            region_folder.exists(),
            "World region folder does not exist!"
        );

        Level {
            root_folder,
            region_folder,
        }
    }

    // /// Read one chunk in the world
    // ///
    // /// Do not use this function if reading many chunks is required, since in case those two chunks which are read separately using `.read_chunk` are in the same region file, it will need to be opened and closed separately for both of them, leading to a performance loss.
    // pub async fn read_chunk(&self, chunk: (i32, i32)) -> Result<ChunkData, WorldError> {
    //     self.read_chunks(vec![chunk])
    //         .await
    //         .pop()
    //         .expect("Read chunks must return a chunk")
    //         .1
    // }

    /// Read many chunks in a world
    /// MUST be called from a tokio runtime thread
    ///
    /// Note: The order of the output chunks will almost never be in the same order as the order of input chunks
    pub async fn read_chunks(
        &self,
        chunks: Vec<(i32, i32)>,
        channel: mpsc::Sender<((i32, i32), Result<ChunkData, WorldError>)>,
    ) {
        chunks
            .into_par_iter()
            .map(|chunk| {
                let region = (
                    ((chunk.0 as f32) / 32.0).floor() as i32,
                    ((chunk.1 as f32) / 32.0).floor() as i32,
                );
                let channel = channel.clone();

                // return different error when file is not found (because that means that the chunks have just not been generated yet)
                let mut region_file = match OpenOptions::new().read(true).open(
                    self.region_folder
                        .join(format!("r.{}.{}.mca", region.0, region.1)),
                ) {
                    Ok(f) => f,
                    Err(err) => match err.kind() {
                        std::io::ErrorKind::NotFound => {
                            let _ = channel.blocking_send((chunk, Err(WorldError::RegionNotFound)));
                            return;
                        }
                        _ => {
                            let _ = channel
                                .blocking_send((chunk, Err(WorldError::IoError(err.kind()))));
                            return;
                        }
                    },
                };

                let mut location_table: [u8; 4096] = [0; 4096];
                let mut timestamp_table: [u8; 4096] = [0; 4096];

                // fill the location and timestamp tables
                {
                    match region_file.read_exact(&mut location_table) {
                        Ok(_) => {}
                        Err(err) => {
                            let _ = channel
                                .blocking_send((chunk, Err(WorldError::IoError(err.kind()))));
                            return;
                        }
                    }
                    match region_file.read_exact(&mut timestamp_table) {
                        Ok(_) => {}
                        Err(err) => {
                            let _ = channel
                                .blocking_send((chunk, Err(WorldError::IoError(err.kind()))));
                            return;
                        }
                    }
                }

                let modulus = |a: i32, b: i32| ((a % b) + b) % b;
                let chunk_x = modulus(chunk.0, 32) as u32;
                let chunk_z = modulus(chunk.1, 32) as u32;
                let channel = channel.clone();
                let table_entry = (chunk_x + chunk_z * 32) * 4;

                let mut offset = vec![0u8];
                offset.extend_from_slice(
                    &location_table[table_entry as usize..table_entry as usize + 3],
                );
                let offset = u32::from_be_bytes(offset.try_into().unwrap()) as u64 * 4096;
                let size = location_table[table_entry as usize + 3] as usize * 4096;

                if offset == 0 && size == 0 {
                    let _ =
                        channel.blocking_send(((chunk.0, chunk.1), Err(WorldError::ChunkNotFound)));
                    return;
                }
                // Read the file using the offset and size
                let mut file_buf = {
                    let seek_result = region_file.seek(std::io::SeekFrom::Start(offset));
                    if seek_result.is_err() {
                        let _ = channel
                            .blocking_send(((chunk.0, chunk.1), Err(WorldError::RegionIsInvalid)));
                        return;
                    }
                    let mut out = vec![0; size];
                    let read_result = region_file.read_exact(&mut out);
                    if read_result.is_err() {
                        let _ = channel
                            .blocking_send(((chunk.0, chunk.1), Err(WorldError::RegionIsInvalid)));
                        return;
                    }
                    out
                };

                // TODO: check checksum to make sure chunk is not corrupted
                let header = file_buf.drain(0..5).collect_vec();

                let compression = match Compression::from_byte(header[4]) {
                    Some(c) => c,
                    None => {
                        let _ = channel.blocking_send((
                            (chunk.0, chunk.1),
                            Err(WorldError::Compression(
                                CompressionError::UnknownCompression,
                            )),
                        ));
                        return;
                    }
                };

                let size = u32::from_be_bytes(header[..4].try_into().unwrap());

                // size includes the compression scheme byte, so we need to subtract 1
                let chunk_data = file_buf.drain(0..size as usize - 1).collect_vec();
                let decompressed_chunk = match Self::decompress_data(compression, chunk_data) {
                    Ok(data) => data,
                    Err(e) => {
                        channel
                            .blocking_send(((chunk.0, chunk.1), Err(WorldError::Compression(e))))
                            .expect("Failed to send Compression error");
                        return;
                    }
                };

                channel
                    .blocking_send((chunk, ChunkData::from_bytes(decompressed_chunk, chunk)))
                    .expect("Error sending decompressed chunk");
            })
            .collect::<Vec<()>>();
    }

    fn decompress_data(
        compression: Compression,
        compressed_data: Vec<u8>,
    ) -> Result<Vec<u8>, CompressionError> {
        match compression {
            Compression::Gzip => {
                let mut z = GzDecoder::new(&compressed_data[..]);
                let mut chunk_data = Vec::new();
                match z.read_to_end(&mut chunk_data) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(CompressionError::GZipError(e));
                    }
                }
                Ok(chunk_data)
            }
            Compression::Zlib => {
                let mut z = ZlibDecoder::new(&compressed_data[..]);
                let mut chunk_data = Vec::new();
                match z.read_to_end(&mut chunk_data) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(CompressionError::ZlibError(e));
                    }
                }
                Ok(chunk_data)
            }
            Compression::None => Ok(compressed_data),
            Compression::LZ4 => todo!(),
        }
    }
}
