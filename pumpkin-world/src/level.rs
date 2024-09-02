use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Read, Seek},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use flate2::{bufread::ZlibDecoder, read::GzDecoder};
use itertools::Itertools;
use pumpkin_core::math::vector2::Vector2;
use rayon::prelude::*;
use thiserror::Error;
use tokio::sync::mpsc;

use crate::{
    chunk::ChunkData,
    world_gen::{get_world_gen, Seed, WorldGenerator},
};

/// The Level represents a single Dimension.
pub struct Level {
    save_file: Option<SaveFile>,
    loaded_chunks: Arc<Mutex<HashMap<Vector2<i32>, Arc<ChunkData>>>>,
    world_gen: Box<dyn WorldGenerator>,
}

struct SaveFile {
    #[allow(dead_code)]
    root_folder: PathBuf,
    region_folder: PathBuf,
}

#[derive(Error, Debug)]
pub enum WorldError {
    // using ErrorKind instead of Error, beacuse the function read_chunks and read_region_chunks is designed to return an error on a per-chunk basis, while std::io::Error does not implement Copy or Clone
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Region is invalid")]
    RegionIsInvalid,
    #[error("The chunk isn't generated yet: {0}")]
    ChunkNotGenerated(ChunkNotGeneratedError),
    #[error("Compression Error")]
    Compression(CompressionError),
    #[error("Error deserializing chunk: {0}")]
    ErrorDeserializingChunk(String),
    #[error("The requested block identifier does not exist")]
    BlockIdentifierNotFound,
    #[error("The requested block state id does not exist")]
    BlockStateIdNotFound,
    #[error("The block is not inside of the chunk")]
    BlockOutsideChunk,
}

#[derive(Error, Debug)]
pub enum ChunkNotGeneratedError {
    #[error("The region file does not exist.")]
    RegionFileMissing,

    #[error("The chunks generation is incomplete.")]
    IncompleteGeneration,

    #[error("Chunk not found.")]
    NotFound,
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
        let world_gen = get_world_gen(Seed(0)); // TODO Read Seed from config.

        if root_folder.exists() {
            let region_folder = root_folder.join("region");
            assert!(
                region_folder.exists(),
                "World region folder does not exist, despite there being a root folder."
            );

            Self {
                world_gen,
                save_file: Some(SaveFile {
                    root_folder,
                    region_folder,
                }),
                loaded_chunks: Arc::new(Mutex::new(HashMap::new())),
            }
        } else {
            log::warn!(
                "Pumpkin currently only supports Superflat World generation. Use a vanilla ./world folder to play in a normal world."
            );

            Self {
                world_gen,
                save_file: None,
                loaded_chunks: Arc::new(Mutex::new(HashMap::new())),
            }
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

    /// Reads/Generates many chunks in a world
    /// MUST be called from a tokio runtime thread
    ///
    /// Note: The order of the output chunks will almost never be in the same order as the order of input chunks
    pub fn fetch_chunks(
        &self,
        chunks: &[Vector2<i32>],
        channel: mpsc::Sender<Result<Arc<ChunkData>, WorldError>>,
        is_alive: bool,
    ) {
        chunks.into_par_iter().for_each(|at| {
            if is_alive {
                return;
            }
            let channel = channel.clone();

            // Check if chunks is already loaded
            let mut loaded_chunks = self.loaded_chunks.lock().unwrap();
            if loaded_chunks.contains_key(at) {
                channel
                    .blocking_send(Ok(loaded_chunks.get(at).unwrap().clone()))
                    .expect("Failed sending ChunkData.");
                return;
            }
            let at = *at;
            let data = match &self.save_file {
                Some(save_file) => {
                    match Self::read_chunk(save_file, at) {
                        Err(WorldError::ChunkNotGenerated(_)) => {
                            // This chunk was not generated yet.
                            Ok(self.world_gen.generate_chunk(at))
                        }
                        // TODO this doesn't warn the user about the error. fix.
                        result => result,
                    }
                }
                None => {
                    // There is no savefile yet -> generate the chunks
                    Ok(self.world_gen.generate_chunk(at))
                }
            }
            .unwrap();
            let data = Arc::new(data);
            channel
                .blocking_send(Ok(data.clone()))
                .expect("Failed sending ChunkData.");
            loaded_chunks.insert(at, data);
        })
    }

    fn read_chunk(save_file: &SaveFile, at: Vector2<i32>) -> Result<ChunkData, WorldError> {
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
                std::io::ErrorKind::NotFound => {
                    WorldError::ChunkNotGenerated(ChunkNotGeneratedError::RegionFileMissing)
                }
                kind => WorldError::IoError(kind),
            })?;

        let mut location_table: [u8; 4096] = [0; 4096];
        let mut timestamp_table: [u8; 4096] = [0; 4096];

        // fill the location and timestamp tables
        region_file
            .read_exact(&mut location_table)
            .map_err(|err| WorldError::IoError(err.kind()))?;
        region_file
            .read_exact(&mut timestamp_table)
            .map_err(|err| WorldError::IoError(err.kind()))?;

        let modulus = |a: i32, b: i32| ((a % b) + b) % b;
        let chunk_x = modulus(at.x, 32) as u32;
        let chunk_z = modulus(at.z, 32) as u32;
        let table_entry = (chunk_x + chunk_z * 32) * 4;

        let mut offset = vec![0u8];
        offset.extend_from_slice(&location_table[table_entry as usize..table_entry as usize + 3]);
        let offset = u32::from_be_bytes(offset.try_into().unwrap()) as u64 * 4096;
        let size = location_table[table_entry as usize + 3] as usize * 4096;

        if offset == 0 && size == 0 {
            return Err(WorldError::ChunkNotGenerated(
                ChunkNotGeneratedError::NotFound,
            ));
        }

        // Read the file using the offset and size
        let mut file_buf = {
            let seek_result = region_file.seek(std::io::SeekFrom::Start(offset));
            if seek_result.is_err() {
                return Err(WorldError::RegionIsInvalid);
            }
            let mut out = vec![0; size];
            let read_result = region_file.read_exact(&mut out);
            if read_result.is_err() {
                return Err(WorldError::RegionIsInvalid);
            }
            out
        };

        // TODO: check checksum to make sure chunk is not corrupted
        let header = file_buf.drain(0..5).collect_vec();

        let compression = match Compression::from_byte(header[4]) {
            Some(c) => c,
            None => {
                return Err(WorldError::Compression(
                    CompressionError::UnknownCompression,
                ))
            }
        };

        let size = u32::from_be_bytes(header[..4].try_into().unwrap());

        // size includes the compression scheme byte, so we need to subtract 1
        let chunk_data = file_buf.drain(0..size as usize - 1).collect_vec();
        let decompressed_chunk =
            Self::decompress_data(compression, chunk_data).map_err(WorldError::Compression)?;

        ChunkData::from_bytes(decompressed_chunk, at)
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
