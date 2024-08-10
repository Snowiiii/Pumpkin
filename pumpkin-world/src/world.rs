use std::{collections::VecDeque, future, io::Read, path::PathBuf, sync::Arc};

use flate2::bufread::ZlibDecoder;
use itertools::Itertools;
use thiserror::Error;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    sync::Mutex,
};

use crate::{chunk::ChunkData, dimension::Dimension};

pub struct Level {
    root_folder: PathBuf,
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
    #[error("Compression scheme not recognised")]
    UnknownCompression,
    #[error("Error while working with zlib compression: {0}")]
    ZlibError(std::io::Error),
    #[error("Error deserializing chunk: {0}")]
    ErrorDeserializingChunk(String),
    #[error("The requested block state id does not exist")]
    BlockStateIdNotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    Gzip,
    Zlib,
    None,
    LZ4,
}

impl Level {
    pub fn from_root_folder(root_folder: PathBuf) -> Self {
        Level { root_folder }
    }

    /// Read one chunk in the world
    ///
    /// Do not use this function if reading many chunks is required, since in case those two chunks which are read seperately using `.read_chunk` are in the same region file, it will need to be opened and closed separately for both of them, leading to a performance loss.
    pub async fn read_chunk(&self, chunk: (i32, i32)) -> Result<ChunkData, WorldError> {
        self.read_chunks(vec![chunk])
            .await
            .pop()
            .expect("Read chunks must return a chunk")
            .1
    }

    /// Read many chunks in a world
    ///
    /// Note: The order of the output chunks will almost never be in the same order as the order of input chunks
    pub async fn read_chunks(
        &self,
        chunks: Vec<(i32, i32)>,
    ) -> Vec<((i32, i32), Result<ChunkData, WorldError>)> {
        futures::future::join_all(
            chunks
                .into_iter()
                // split chunks into their corresponding region files to be able to read all of them at once, instead of reopening the file multiple times
                .chunk_by(|chunk| {
                    (
                        ((chunk.0 as f32) / 32.0).floor() as i32,
                        ((chunk.1 as f32) / 32.0).floor() as i32,
                    )
                })
                .into_iter()
                .map(|(region, chunk_vec)| {
                    let mut path = self.root_folder.clone();
                    path.push("region");
                    path.push(format!("r.{}.{}.mca", region.0, region.1));
                    self.read_region_chunks(path, chunk_vec.collect_vec())
                }),
        )
        .await
        .into_iter()
        .flatten()
        .collect_vec()
    }
    async fn read_region_chunks(
        &self,
        region_file: PathBuf,
        chunks: Vec<(i32, i32)>,
    ) -> Vec<((i32, i32), Result<ChunkData, WorldError>)> {
        // dbg!(at);
        println!(
            "Getting chunks {:?}, from region file {}",
            &chunks,
            region_file.to_str().unwrap_or("")
        );
        // return different error when file is not found (because that means that the chunks have just not been generated yet)
        let mut region_file = match File::open(region_file).await {
            Ok(f) => f,
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    return chunks
                        .into_iter()
                        .map(|c| (c, Err(WorldError::RegionNotFound)))
                        .collect_vec()
                }
                _ => {
                    return chunks
                        .into_iter()
                        .map(|c| (c, Err(WorldError::IoError(err.kind()))))
                        .collect_vec()
                }
            },
        };

        let mut location_table: [u8; 4096] = [0; 4096];
        let mut timestamp_table: [u8; 4096] = [0; 4096];

        // fill the location and timestamp tables
        {
            match region_file.read_exact(&mut location_table).await {
                Ok(_) => {}
                Err(err) => {
                    return chunks
                        .into_iter()
                        .map(|c| (c, Err(WorldError::IoError(err.kind()))))
                        .collect_vec()
                }
            }
            match region_file.read_exact(&mut timestamp_table).await {
                Ok(_) => {}
                Err(err) => {
                    return chunks
                        .into_iter()
                        .map(|c| (c, Err(WorldError::IoError(err.kind()))))
                        .collect_vec()
                }
            }
        }
        // println!("Location table: {:?}", &location_table);

        // wrap file with arc mutex to allow for multithreading
        let region_file = Arc::new(Mutex::new(region_file));
        futures::future::join_all(chunks.into_iter().map(|(old_chunk_x, old_chunk_z)| {
            let region_file = region_file.clone();
            let modulus = |a: i32, b: i32| ((a % b) + b) % b;
            let chunk_x = modulus(old_chunk_x, 32) as u32;
            let chunk_z = modulus(old_chunk_z, 32) as u32;
            async move {
                let table_entry = (chunk_x + chunk_z * 32) * 4;

                let mut offset = vec![0u8];
                offset.extend_from_slice(
                    &location_table[table_entry as usize..table_entry as usize + 3],
                );
                let offset = u32::from_be_bytes(offset.try_into().unwrap()) as u64 * 4096;
                let size = location_table[table_entry as usize + 3] as usize * 4096;

                if offset == 0 && size == 0 {
                    return ((old_chunk_x, old_chunk_z), Err(WorldError::ChunkNotFound));
                }
                // Read the file using the offset and size
                let mut file_buf = {
                    let mut region_file = region_file.lock().await;
                    let seek_result = region_file.seek(std::io::SeekFrom::Start(offset)).await;
                    if seek_result.is_err() {
                        return ((old_chunk_x, old_chunk_z), Err(WorldError::RegionIsInvalid));
                    }
                    let mut out = vec![0; size];
                    let read_result = region_file.read_exact(&mut out).await;
                    if read_result.is_err() {
                        return ((old_chunk_x, old_chunk_z), Err(WorldError::RegionIsInvalid));
                    }
                    out
                };

                // TODO: check checksum to make sure chunk is not corrupted
                let header = file_buf.drain(0..5).collect_vec();

                let compression = match header[4] {
                    1 => Compression::Gzip,
                    2 => Compression::Zlib,
                    3 => Compression::None,
                    4 => Compression::LZ4,
                    _ => return ((old_chunk_x, old_chunk_z), Err(WorldError::RegionIsInvalid)),
                };

                match compression {
                    Compression::Zlib => {}
                    _ => panic!(), // TODO: support other compression types
                }

                let size = u32::from_be_bytes(header[0..4].try_into().unwrap());

                let chunk_data = file_buf.drain(0..size as usize).collect_vec();

                let mut z = ZlibDecoder::new(&chunk_data[..]);
                let mut chunk_data = Vec::new();
                match z.read_to_end(&mut chunk_data) {
                    Ok(_) => {}
                    Err(err) => {
                        return ((old_chunk_x, old_chunk_z), Err(WorldError::ZlibError(err)))
                    }
                }
                // TODO: remove
                // File::create_new(format!("./test-{}.{}.nbt", old_chunk_x, old_chunk_z))
                // .await
                // .unwrap()
                // .write_all(&chunk_data)
                // .await
                // .unwrap();

                (
                    (old_chunk_x, old_chunk_z),
                    ChunkData::from_bytes(chunk_data, (old_chunk_x, old_chunk_z)),
                )
            }
        }))
        .await
        .into_iter()
        .collect_vec()
    }
}

#[test]
fn wawa() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        Dimension::OverWorld
            .into_level(
                "C:\\Users\\lukza\\Desktop\\code\\rust\\vanilla_mc_server\\world"
                    .parse()
                    .unwrap(),
            )
            .read_chunk((0, 0))
            .await
            .unwrap();
    });
}
