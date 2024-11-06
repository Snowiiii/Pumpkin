use std::{path::PathBuf, sync::Arc};

use dashmap::{DashMap, Entry};
use num_traits::Zero;
use pumpkin_core::math::vector2::Vector2;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tokio::{
    sync::{mpsc, RwLock},
    task::JoinHandle,
};

use crate::{
    chunk::{
        anvil::AnvilChunkFormat, ChunkData, ChunkParsingError, ChunkReader, ChunkReadingError,
        ChunkWriter,
    },
    world_gen::{get_world_gen, Seed, WorldGenerator},
};

pub type ConcurrentChunkResult = Vec<(Vector2<i32>, JoinHandle<()>)>;

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
    level_folder: LevelFolder,
    loaded_chunks: Arc<DashMap<Vector2<i32>, Arc<RwLock<ChunkData>>>>,
    chunk_watchers: Arc<DashMap<Vector2<i32>, usize>>,
    chunk_reader: Arc<dyn ChunkReader>,
    chunk_writer: Arc<dyn ChunkWriter>,
    world_gen: Arc<dyn WorldGenerator>,
}

#[derive(Clone)]
pub struct LevelFolder {
    pub root_folder: PathBuf,
    pub region_folder: PathBuf,
}

impl Level {
    pub fn from_root_folder(root_folder: PathBuf) -> Self {
        // TODO Read Seed from config.
        let world_gen = get_world_gen(Seed(0)).into();
        // Check if region folder exists, if not lets make one
        let region_folder = root_folder.join("region");
        if !region_folder.exists() {
            std::fs::create_dir_all(&region_folder).expect("Failed to create Region folder");
        }
        Self {
            world_gen,
            level_folder: LevelFolder {
                root_folder,
                region_folder,
            },
            chunk_reader: Arc::new(AnvilChunkFormat::new()),
            chunk_writer: Arc::new(AnvilChunkFormat::new()),
            loaded_chunks: Arc::new(DashMap::new()),
            chunk_watchers: Arc::new(DashMap::new()),
        }
    }

    pub fn get_block() {}

    pub fn loaded_chunk_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    pub fn list_cached(&self) {
        for entry in self.loaded_chunks.iter() {
            log::debug!("In map: {:?}", entry.key());
        }
    }

    /// Marks chunks as "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was not watching
    /// before
    pub fn mark_chunks_as_newly_watched(&self, chunks: &[Vector2<i32>]) {
        chunks.par_iter().for_each(|chunk| {
            self.mark_chunk_as_newly_watched(*chunk);
        });
    }

    pub fn mark_chunk_as_newly_watched(&self, chunk: Vector2<i32>) {
        match self.chunk_watchers.entry(chunk) {
            Entry::Occupied(mut occupied) => {
                let value = occupied.get_mut();
                if let Some(new_value) = value.checked_add(1) {
                    *value = new_value;
                    //log::debug!("Watch value for {:?}: {}", chunk, value);
                } else {
                    log::error!("Watching overflow on chunk {:?}", chunk);
                }
            }
            Entry::Vacant(vacant) => {
                vacant.insert(1);
            }
        }
    }

    /// Marks chunks no longer "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was watching before
    pub fn mark_chunks_as_not_watched(&self, chunks: &[Vector2<i32>]) -> Vec<Vector2<i32>> {
        chunks
            .par_iter()
            .filter(|chunk| self.mark_chunk_as_not_watched(**chunk))
            .map(|chunk| *chunk)
            .collect()
    }

    /// Returns whether the chunk should be removed from memory
    pub fn mark_chunk_as_not_watched(&self, chunk: Vector2<i32>) -> bool {
        match self.chunk_watchers.entry(chunk) {
            Entry::Occupied(mut occupied) => {
                let value = occupied.get_mut();
                *value = value.saturating_sub(1);
                if *value == 0 {
                    occupied.remove_entry();
                    true
                } else {
                    false
                }
            }
            Entry::Vacant(_) => {
                // This can be:
                // - Player disconnecting before all packets have been sent
                // - Player moving so fast that the chunk leaves the render distance before it
                // is loaded into memory
                log::error!(
                    "Marking a chunk as not watched, but was vacant! ({:?})",
                    chunk
                );
                false
            }
        }
    }

    pub fn should_pop_chunk(&self, chunk: &Vector2<i32>) -> bool {
        if let Some(entry) = self.chunk_watchers.get(chunk) {
            if entry.value().is_zero() {
                self.chunk_watchers.remove(chunk);
            }
        }

        self.chunk_watchers.get(chunk).is_none()
    }

    pub fn clean_chunks(&self, chunks: &[Vector2<i32>]) {
        chunks.par_iter().for_each(|chunk_pos| {
            //log::debug!("Unloading {:?}", chunk_pos);
            // Save all chunks
            if let Some(data) = self.loaded_chunks.remove(chunk_pos) {
                if let Err(error) = self.chunk_writer.write_chunk(
                    &data.1.blocking_read(),
                    &self.level_folder,
                    &data.0,
                ) {
                    log::error!("Failed writing Chunk to disk {}", error.to_string());
                }
            };
        });
    }

    pub fn clean_memory(&self, chunks_to_check: &[Vector2<i32>]) {
        chunks_to_check.par_iter().for_each(|chunk| {
            if let Some(entry) = self.chunk_watchers.get(chunk) {
                if entry.value().is_zero() {
                    self.chunk_watchers.remove(chunk);
                }
            }

            if self.chunk_watchers.get(chunk).is_none() {
                self.loaded_chunks.remove(chunk);
            }
        });
        self.loaded_chunks.shrink_to_fit();
        self.chunk_watchers.shrink_to_fit();
    }

    fn load_chunk_from_save(
        chunk_reader: Arc<dyn ChunkReader>,
        level_folder: &LevelFolder,
        chunk_pos: Vector2<i32>,
    ) -> Result<Option<Arc<RwLock<ChunkData>>>, ChunkReadingError> {
        match chunk_reader.read_chunk(level_folder, &chunk_pos) {
            Ok(data) => Ok(Some(Arc::new(RwLock::new(data)))),
            Err(
                ChunkReadingError::ChunkNotExist
                | ChunkReadingError::ParsingError(ChunkParsingError::ChunkNotGenerated),
            ) => {
                // This chunk was not generated yet.
                Ok(None)
            }
            Err(err) => Err(err),
        }
    }

    /// Reads/Generates many chunks in a world
    /// MUST be called from a tokio runtime thread
    ///
    /// Note: The order of the output chunks will almost never be in the same order as the order of input chunks
    pub fn fetch_chunks(
        &self,
        chunks: &[Vector2<i32>],
        channel: mpsc::Sender<Arc<RwLock<ChunkData>>>,
    ) -> ConcurrentChunkResult {
        chunks
            .iter()
            .map(|at| {
                let channel = channel.clone();
                let loaded_chunks = self.loaded_chunks.clone();
                let chunk_reader = self.chunk_reader.clone();
                let chunk_writer = self.chunk_writer.clone();
                let level_folder = self.level_folder.clone();
                let world_gen = self.world_gen.clone();
                let chunk_pos = *at;

                let join_handle = tokio::spawn(async move {
                    let chunk = loaded_chunks
                        .get(&chunk_pos)
                        .map(|entry| entry.value().clone())
                        .unwrap_or_else(|| {
                            let loaded_chunk = match Self::load_chunk_from_save(
                                chunk_reader,
                                &level_folder,
                                chunk_pos,
                            ) {
                                Ok(chunk) => {
                                    // Save new Chunk
                                    if let Some(chunk) = &chunk {
                                        if let Err(error) = chunk_writer.write_chunk(
                                            &chunk.blocking_read(),
                                            &level_folder,
                                            &chunk_pos,
                                        ) {
                                            log::error!(
                                                "Failed writing Chunk to disk {}",
                                                error.to_string()
                                            );
                                        };
                                    }
                                    chunk
                                }
                                Err(err) => {
                                    log::error!(
                                        "Failed to read chunk (regenerating) {:?}: {:?}",
                                        chunk_pos,
                                        err
                                    );
                                    None
                                }
                            }
                            .unwrap_or_else(|| {
                                Arc::new(RwLock::new(world_gen.generate_chunk(chunk_pos)))
                            });
                            // Save Chunk

                            if let Some(data) = loaded_chunks.get(&chunk_pos) {
                                // Another thread populated in between the previous check and now
                                // We did work, but this is basically like a cache miss, not much we
                                // can do about it
                                data.value().clone()
                            } else {
                                loaded_chunks.insert(chunk_pos, loaded_chunk.clone());
                                loaded_chunk
                            }
                        });

                    let _ = channel
                        .send(chunk)
                        .await
                        .inspect_err(|err| log::error!("unable to send chunk to channel: {}", err));
                });

                (*at, join_handle)
            })
            .collect()
    }
}
