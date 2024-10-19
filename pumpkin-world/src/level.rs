use std::{collections::HashMap, path::PathBuf, sync::Arc};

use parking_lot::{Mutex, RwLock};
use pumpkin_core::math::vector2::Vector2;
use rayon::prelude::*;
use tokio::sync::mpsc;

use crate::{
    chunk::{
        anvil::AnvilChunkReader, ChunkData, ChunkParsingError, ChunkReader, ChunkReadingError,
    },
    world_gen::{get_world_gen, Seed, WorldGenerator},
};

/// The `Level` module provides functionality for working with chunks within or outside a Minecraft world.
///
/// Key features include:
///
/// - **Chunk Loading:** Efficiently loads chunks from disk.
/// - **Chunk Caching:** Stores accessed chunks in memory for faster access.
/// - **Chunk Generation:** Generates new chunks on-demand using a specified `WorldGenerator`.
///
/// For more details on world generation, refer to the `WorldGenerator` module.
pub struct Level {
    save_file: Option<SaveFile>,
    loaded_chunks: Arc<RwLock<HashMap<Vector2<i32>, Arc<ChunkData>>>>,
    chunk_watchers: Arc<Mutex<HashMap<Vector2<i32>, usize>>>,
    chunk_reader: Box<dyn ChunkReader>,
    world_gen: Box<dyn WorldGenerator>,
}

pub struct SaveFile {
    #[expect(dead_code)]
    root_folder: PathBuf,
    pub region_folder: PathBuf,
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
                chunk_reader: Box::new(AnvilChunkReader::new()),
                loaded_chunks: Arc::new(RwLock::new(HashMap::new())),
                chunk_watchers: Arc::new(Mutex::new(HashMap::new())),
            }
        } else {
            log::warn!(
                "Pumpkin currently only supports Superflat World generation. Use a vanilla ./world folder to play in a normal world."
            );

            Self {
                world_gen,
                save_file: None,
                chunk_reader: Box::new(AnvilChunkReader::new()),
                loaded_chunks: Arc::new(RwLock::new(HashMap::new())),
                chunk_watchers: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    pub fn get_block() {}

    /// Marks chunks as "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was not watching
    /// before
    pub fn mark_chunk_as_newly_watched(&self, chunks: &[Vector2<i32>]) {
        let mut watchers = self.chunk_watchers.lock();
        for chunk in chunks {
            match watchers.entry(*chunk) {
                std::collections::hash_map::Entry::Occupied(mut occupied) => {
                    let value = occupied.get_mut();
                    *value = value.saturating_add(1);
                }
                std::collections::hash_map::Entry::Vacant(vacant) => {
                    vacant.insert(1);
                }
            }
        }
    }

    /// Marks chunks no longer "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was watching before
    pub fn mark_chunk_as_not_watched_and_clean(&self, chunks: &[Vector2<i32>]) {
        let dropped_chunks = {
            let mut watchers = self.chunk_watchers.lock();
            chunks
                .iter()
                .filter(|chunk| match watchers.entry(**chunk) {
                    std::collections::hash_map::Entry::Occupied(mut occupied) => {
                        let value = occupied.get_mut();
                        *value = value.saturating_sub(1);
                        if *value == 0 {
                            occupied.remove_entry();
                            true
                        } else {
                            false
                        }
                    }
                    std::collections::hash_map::Entry::Vacant(_) => {
                        log::error!(
                            "Marking a chunk as not watched, but was vacant! ({:?})",
                            chunk
                        );
                        false
                    }
                })
                .collect::<Vec<_>>()
        };
        let mut loaded_chunks = self.loaded_chunks.write();
        let dropped_chunk_data = dropped_chunks
            .iter()
            .filter_map(|chunk| {
                log::debug!("Unloading chunk {:?}", chunk);
                loaded_chunks.remove_entry(*chunk)
            })
            .collect();
        self.write_chunks(dropped_chunk_data);
    }

    pub fn write_chunks(&self, _chunks_to_write: Vec<(Vector2<i32>, Arc<ChunkData>)>) {
        //TODO
    }

    /// Reads/Generates many chunks in a world
    /// MUST be called from a tokio runtime thread
    ///
    /// Note: The order of the output chunks will almost never be in the same order as the order of input chunks

    pub fn fetch_chunks(&self, chunks: &[Vector2<i32>], channel: mpsc::Sender<Arc<ChunkData>>) {
        chunks.into_par_iter().for_each(|at| {
            let channel = channel.clone();

            let maybe_chunk = {
                let loaded_chunks = self.loaded_chunks.read();
                loaded_chunks.get(at).cloned()
            }
            .or_else(|| {
                let chunk_data = match &self.save_file {
                    Some(save_file) => {
                        match self.chunk_reader.read_chunk(save_file, at) {
                            Ok(data) => Ok(Arc::new(data)),
                            Err(
                                ChunkReadingError::ChunkNotExist
                                | ChunkReadingError::ParsingError(
                                    ChunkParsingError::ChunkNotGenerated,
                                ),
                            ) => {
                                // This chunk was not generated yet.
                                let chunk = Arc::new(self.world_gen.generate_chunk(*at));
                                Ok(chunk)
                            }
                            Err(err) => Err(err),
                        }
                    }
                    None => {
                        // There is no savefile yet -> generate the chunks
                        let chunk = Arc::new(self.world_gen.generate_chunk(*at));
                        Ok(chunk)
                    }
                };
                match chunk_data {
                    Ok(data) => Some(data),
                    Err(err) => {
                        // TODO: Panic here?
                        log::warn!("Failed to read chunk {:?}: {:?}", at, err);
                        None
                    }
                }
            });
            match maybe_chunk {
                Some(chunk) => {
                    channel
                        .blocking_send(chunk.clone())
                        .expect("Failed sending ChunkData.");
                }
                None => {
                    log::error!("Unable to send chunk {:?}!", at);
                }
            };
        })
    }
}
