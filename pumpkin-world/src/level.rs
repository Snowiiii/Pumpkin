use std::{collections::HashMap, path::PathBuf, sync::Arc};

use parking_lot::Mutex;
use pumpkin_core::math::vector2::Vector2;
use rayon::prelude::*;
use tokio::sync::mpsc;

use crate::{
    chunk::{anvil::AnvilChunkReader, ChunkData, ChunkReader, ChunkReadingError},
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
    loaded_chunks: Arc<Mutex<HashMap<Vector2<i32>, Arc<ChunkData>>>>,
    chunk_reader: Box<dyn ChunkReader>,
    world_gen: Box<dyn WorldGenerator>,
}

pub struct SaveFile {
    pub root_folder: PathBuf,
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
                loaded_chunks: Arc::new(Mutex::new(HashMap::new())),
            }
        } else {
            log::warn!(
                "Pumpkin currently only supports Superflat World generation. Use a vanilla ./world folder to play in a normal world."
            );

            Self {
                world_gen,
                save_file: None,
                chunk_reader: Box::new(AnvilChunkReader::new()),
                loaded_chunks: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    pub fn get_block() {}

    /// Reads/Generates many chunks in a world
    /// MUST be called from a tokio runtime thread
    ///
    /// Note: The order of the output chunks will almost never be in the same order as the order of input chunks
    pub fn fetch_chunks(
        &mut self,
        chunks: &[Vector2<i32>],
        channel: mpsc::Sender<Arc<ChunkData>>,
        is_alive: bool,
    ) {
        chunks.into_par_iter().for_each(|at| {
            if is_alive {
                return;
            }
            let mut loaded_chunks = self.loaded_chunks.lock();
            let channel = channel.clone();

            // Check if chunks is already loaded
            if loaded_chunks.contains_key(at) {
                channel
                    .blocking_send(loaded_chunks.get(at).unwrap().clone())
                    .expect("Failed sending ChunkData.");
                return;
            }
            let at = *at;
            let data = match &self.save_file {
                Some(save_file) => {
                    match self.chunk_reader.read_chunk(save_file, at) {
                        Err(ChunkReadingError::ChunkNotExist) => {
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
                .blocking_send(data.clone())
                .expect("Failed sending ChunkData.");
            loaded_chunks.insert(at, data);
        })
    }
}
