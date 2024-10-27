use std::{path::PathBuf, sync::Arc};

use dashmap::{DashMap, Entry};
use pumpkin_core::math::vector2::Vector2;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tokio::sync::{mpsc, RwLock};

use crate::{
    chunk::{
        anvil::AnvilChunkReader, ChunkData, ChunkParsingError, ChunkReader, ChunkReadingError,
    },
    lighting::manager::LevelLightManager,
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
    loaded_chunks: Arc<DashMap<Vector2<i32>, Arc<RwLock<ChunkData>>>>,
    chunk_watchers: Arc<DashMap<Vector2<i32>, usize>>,
    chunk_reader: Arc<dyn ChunkReader>,
    world_gen: Arc<dyn WorldGenerator>,
    light_manager: Arc<LevelLightManager>,
}

#[derive(Clone)]
pub struct SaveFile {
    #[expect(dead_code)]
    root_folder: PathBuf,
    pub region_folder: PathBuf,
}

impl Level {
    pub fn from_root_folder(root_folder: PathBuf) -> Self {
        let world_gen = get_world_gen(Seed(0)).into(); // TODO Read Seed from config.
        if root_folder.exists() {
            let region_folder = root_folder.join("region");
            assert!(
                region_folder.exists(),
                "World region folder does not exist, despite there being a root folder."
            );

            let loaded_chunks = Arc::new(DashMap::new());

            Self {
                world_gen,
                save_file: Some(SaveFile {
                    root_folder,
                    region_folder,
                }),
                chunk_reader: Arc::new(AnvilChunkReader::new()),
                chunk_watchers: Arc::new(DashMap::new()),
                light_manager: Arc::new(LevelLightManager::new(loaded_chunks.clone())),
                loaded_chunks,
            }
        } else {
            log::warn!(
                "Pumpkin currently only supports Superflat World generation. Use a vanilla ./world folder to play in a normal world."
            );

            let loaded_chunks = Arc::new(DashMap::new());

            Self {
                world_gen,
                save_file: None,
                chunk_reader: Arc::new(AnvilChunkReader::new()),
                chunk_watchers: Arc::new(DashMap::new()),
                light_manager: Arc::new(LevelLightManager::new(loaded_chunks.clone())),
                loaded_chunks,
            }
        }
    }

    pub fn get_block() {}

    pub fn loaded_chunk_count(&self) -> usize {
        self.loaded_chunks.len()
    }

    /// Marks chunks as "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was not watching
    /// before
    pub fn mark_chunks_as_newly_watched(&self, chunks: &[Vector2<i32>]) {
        chunks
            .par_iter()
            .for_each(|chunk| match self.chunk_watchers.entry(*chunk) {
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
            });
    }

    /// Marks chunks no longer "watched" by a unique player. When no players are watching a chunk,
    /// it is removed from memory. Should only be called on chunks the player was watching before
    pub async fn mark_chunk_as_not_watched_and_clean(&self, chunks: &[Vector2<i32>]) {
        chunks
            .par_iter()
            .filter(|chunk| match self.chunk_watchers.entry(**chunk) {
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
                    log::error!(
                        "Marking a chunk as not watched, but was vacant! ({:?})",
                        chunk
                    );
                    false
                }
            })
            .for_each(|chunk_pos| {
                //log::debug!("Unloading {:?}", chunk_pos);
                if let Some(data) = self.loaded_chunks.remove(chunk_pos) {
                    self.write_chunk(data);
                };
            });
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
    /// MUST be called from a tokio runtime thread
    ///
    /// Note: The order of the output chunks will almost never be in the same order as the order of input chunks
    pub async fn fetch_chunks(
        &self,
        chunks: &[Vector2<i32>],
        channel: mpsc::Sender<Arc<RwLock<ChunkData>>>,
    ) {
        chunks.iter().for_each(|at| {
            let channel = channel.clone();
            let loaded_chunks = self.loaded_chunks.clone();
            let chunk_reader = self.chunk_reader.clone();
            let save_file = self.save_file.clone();
            let world_gen = self.world_gen.clone();
            let light_manager = self.light_manager.clone();
            let chunk_pos = *at;

            tokio::spawn(async move {
                let chunk = loaded_chunks
                    .get(&chunk_pos)
                    .map(|entry| entry.value().clone());
                let chunk = match chunk {
                    Some(chunk) => chunk,
                    None => {
                        let loaded_chunk = save_file
                            .and_then(|save_file| {
                                match Self::load_chunk_from_save(chunk_reader, save_file, chunk_pos)
                                {
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

                            light_manager.initialize_lighting(chunk_pos).await;
                            loaded_chunk
                        }
                    }
                };

                let _ = channel
                    .send(chunk)
                    .await
                    .inspect_err(|err| log::error!("unable to send chunk to channel: {}", err));
            });
        })
    }
}
