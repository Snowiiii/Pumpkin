use std::{path::PathBuf, sync::Arc};

use dashmap::{DashMap, Entry};
use num_traits::Zero;
use pumpkin_config::BASIC_CONFIG;
use pumpkin_core::math::vector2::Vector2;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tokio::{
    runtime::Handle,
    sync::{mpsc, RwLock},
};

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
    pub seed: Seed,
    save_file: Option<SaveFile>,
    loaded_chunks: Arc<DashMap<Vector2<i32>, Arc<RwLock<ChunkData>>>>,
    chunk_watchers: Arc<DashMap<Vector2<i32>, usize>>,
    chunk_reader: Arc<dyn ChunkReader>,
    world_gen: Arc<dyn WorldGenerator>,
}

#[derive(Clone)]
pub struct SaveFile {
    pub root_folder: PathBuf,
    pub region_folder: PathBuf,
}

fn get_or_create_seed() -> Seed {
    // TODO: if there is a seed in the config (!= 0) use it. Otherwise make a random one
    Seed::from(BASIC_CONFIG.seed.as_str())
}

impl Level {
    pub fn from_root_folder(root_folder: PathBuf) -> Self {
        // If we are using an already existing world we want to read the seed from the level.dat, If not we want to check if there is a seed in the config, if not lets create a random one
        if root_folder.exists() {
            let region_folder = root_folder.join("region");
            assert!(
                region_folder.exists(),
                "World region folder does not exist, despite there being a root folder."
            );
            // TODO: read seed from level.dat
            let seed = get_or_create_seed();
            let world_gen = get_world_gen(seed).into(); // TODO Read Seed from config.

            Self {
                seed,
                world_gen,
                save_file: Some(SaveFile {
                    root_folder,
                    region_folder,
                }),
                chunk_reader: Arc::new(AnvilChunkReader::new()),
                loaded_chunks: Arc::new(DashMap::new()),
                chunk_watchers: Arc::new(DashMap::new()),
            }
        } else {
            let seed = get_or_create_seed();
            let world_gen = get_world_gen(seed).into(); // TODO Read Seed from config.
            Self {
                seed,
                world_gen,
                save_file: None,
                chunk_reader: Arc::new(AnvilChunkReader::new()),
                loaded_chunks: Arc::new(DashMap::new()),
                chunk_watchers: Arc::new(DashMap::new()),
            }
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
        chunks.iter().for_each(|chunk| {
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
            .iter()
            .filter(|chunk| self.mark_chunk_as_not_watched(**chunk))
            .copied()
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
                true
            }
        }
    }

    pub fn clean_chunks(&self, chunks: &[Vector2<i32>]) {
        chunks.iter().for_each(|chunk_pos| {
            //log::debug!("Unloading {:?}", chunk_pos);
            self.clean_chunk(chunk_pos);
        });
    }

    pub fn clean_chunk(&self, chunk: &Vector2<i32>) {
        if let Some(data) = self.loaded_chunks.remove(chunk) {
            self.write_chunk(data);
        }
    }

    pub fn is_chunk_watched(&self, chunk: &Vector2<i32>) -> bool {
        self.chunk_watchers.get(chunk).is_some()
    }

    pub fn clean_memory(&self, chunks_to_check: &[Vector2<i32>]) {
        chunks_to_check.iter().for_each(|chunk| {
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

    pub fn write_chunk(&self, _chunk_to_write: (Vector2<i32>, Arc<RwLock<ChunkData>>)) {
        //TODO
    }

    fn load_chunk_from_save(
        chunk_reader: Arc<dyn ChunkReader>,
        save_file: SaveFile,
        chunk_pos: Vector2<i32>,
    ) -> Result<Option<Arc<RwLock<ChunkData>>>, ChunkReadingError> {
        match chunk_reader.read_chunk(&save_file, &chunk_pos) {
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
    /// Note: The order of the output chunks will almost never be in the same order as the order of input chunks
    pub fn fetch_chunks(
        &self,
        chunks: &[Vector2<i32>],
        channel: mpsc::Sender<Arc<RwLock<ChunkData>>>,
        rt: &Handle,
    ) {
        chunks.par_iter().for_each(|at| {
            let channel = channel.clone();
            let loaded_chunks = self.loaded_chunks.clone();
            let chunk_reader = self.chunk_reader.clone();
            let save_file = self.save_file.clone();
            let world_gen = self.world_gen.clone();
            let chunk_pos = *at;

            let chunk = loaded_chunks
                .get(&chunk_pos)
                .map(|entry| entry.value().clone())
                .unwrap_or_else(|| {
                    let loaded_chunk = save_file
                        .and_then(|save_file| {
                            match Self::load_chunk_from_save(chunk_reader, save_file, chunk_pos) {
                                Ok(chunk) => chunk,
                                Err(err) => {
                                    log::error!(
                                        "Failed to read chunk (regenerating) {:?}: {:?}",
                                        chunk_pos,
                                        err
                                    );
                                    None
                                }
                            }
                        })
                        .unwrap_or_else(|| {
                            Arc::new(RwLock::new(world_gen.generate_chunk(chunk_pos)))
                        });

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

            rt.spawn(async move {
                let _ = channel
                    .send(chunk)
                    .await
                    .inspect_err(|err| log::error!("unable to send chunk to channel: {}", err));
            });
        });
    }
}
