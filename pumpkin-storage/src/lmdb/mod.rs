use std::{num::NonZeroUsize, ops::Deref, path::PathBuf, sync::{atomic::{AtomicUsize, Ordering}, Arc}};

use crossbeam::sync::ShardedLock;
use encoding::Speedy;
use heed::{byteorder::LE, types::U64, Env, EnvFlags, EnvOpenOptions};
use pumpkin_world::chunk::ChunkData;
use rayon::ThreadPoolBuilder;
use threadpool::{LMDB_INCREMENT_RUNTIME, LMDB_MAP_RUNTIME_SIZE, LMDB_READER_THREADPOOL, LMDB_RESIZE_HANDLE, LMDB_WRITER_THREAD};
use xxhash_rust::xxh3::Xxh3;

use crate::ChunkStorage;

mod encoding;
mod threadpool;

// LMDB parameters constants
const LMDB_MAX_DATABASE: u32 = 1;
const LMDB_MIN_PAGE_SIZE: usize = 300usize.pow(3); // 300MiB
const LMDB_PAGE_SIZE_INCREMENT: usize = 300*1024usize.pow(2); // 300 MiB

// Tables names
const LMDB_TABLE_CHUNK_STORAGE: &str = "chunks";

#[derive(Clone)]
/// A chunk storage over LMDB
pub struct LMDBStorage {
    /// Shared handle of LMDB
    env: Env,
}

/// Configurable options for LMDB.
pub struct LMDBOpenOptions {
    /// Path towards world folder
    path: PathBuf,
    /// Minimal page size
    page_size: Option<NonZeroUsize>,
    /// Page increment
    page_increment: Option<NonZeroUsize>,
}

impl Deref for LMDBStorage {
    type Target = Env;

    fn deref(&self) -> &Self::Target {
        &self.env
    }
}

/// LMDB will show a constant growth over time.
pub fn page_calculation(old_size: usize) -> usize {
    old_size + *LMDB_INCREMENT_RUNTIME.get().unwrap()
}

impl ChunkStorage for LMDBStorage {
    type Error = heed::Error;

    type OpenOptions = LMDBOpenOptions;
    
    /// Start LMDB and initialize all runtime OnceLock for runtime operations.
    async fn start(options: LMDBOpenOptions) -> Result<Self, Self::Error> {
        
        // TODO: dynamic thread count (num_cpu equation)
        let mut env_options = EnvOpenOptions::new();
        let env = unsafe {
            env_options
                .max_dbs(LMDB_MAX_DATABASE)
                .map_size(options.page_size.map(Into::<usize>::into).unwrap_or(LMDB_MIN_PAGE_SIZE))
                .max_readers(8);
            
            #[cfg(not(target_os = "windows"))]
            env_options
                .flags(EnvFlags::WRITE_MAP);
            
            env_options.open(options.path)
        }?;
        
        // Initialize page increment size
        let _ = LMDB_INCREMENT_RUNTIME.get_or_init(|| {
            options.page_increment.map(Into::<usize>::into).unwrap_or(LMDB_PAGE_SIZE_INCREMENT)
        });
        
        // Initialize runtime map size
        let _ = LMDB_MAP_RUNTIME_SIZE.get_or_init(|| {
            let real_size = env.real_disk_size().expect("Unable to read disk file size") as usize;
            let increment = *LMDB_INCREMENT_RUNTIME.get().unwrap();
            Arc::new(
                AtomicUsize::new(
                    (real_size / increment) * (increment+1)
                )
            )
        });
        
        // Resize environment to runtime page
        unsafe { env.resize(LMDB_MAP_RUNTIME_SIZE.get().unwrap().load(Ordering::Relaxed)) }?;
        
        // Initialize resize handle
        let _ = LMDB_RESIZE_HANDLE.get_or_init(|| {
            ShardedLock::new(())
        });
        
        // Initialize writer thread
        let _ = LMDB_WRITER_THREAD.get_or_init(|| {
            ThreadPoolBuilder::new().num_threads(1).build().expect("Unable to build LMDB rayon writer thread")
        });
        
        // Initialize readers threadpool
        let _ = LMDB_READER_THREADPOOL.get_or_init(|| {
           ThreadPoolBuilder::new().num_threads(8).build().expect("Unable to build LMDB rayon readers threadpool.")
        });
        
        // Check if world exist
        {
            let mut rw_tx = env.write_txn()?;
            let database = env.open_database::<U64<LE>, Speedy<ChunkData>>(&rw_tx, Some(LMDB_TABLE_CHUNK_STORAGE))?;
            if database.is_none() {
                // LOG: warn!("No chunks table found. Recreating one")
                env.create_database::<U64<LE>, Speedy<ChunkData>>(&mut rw_tx, Some(LMDB_TABLE_CHUNK_STORAGE))?;
            }
        }
        
        Ok(LMDBStorage { env })
    }

    /// Close LMDB database
    /// 
    /// # Blocking
    /// 
    /// This method will block until all copies of the env have disappeared. It is 
    /// caller responsibility to ensure that this function is call when any shared state
    /// has been dismissed.
    async fn close(self) {
        let event = self.env.prepare_for_closing();
        event.wait();
    }

    /// Get a chunk with its coordinate and dimension from storage (if it exists).
    async fn get_chunk(&self, x: i32, z: i32, dimension: &'static str) -> Result<Option<ChunkData>, Self::Error> {
        let db = self.env.clone();
        
        Self::spawn_read(move || {
            let key = calculate_chunk_key(x,z,dimension);
            let ro_tx = db.read_txn()?;
            let chunks = db.open_database::<U64<LE>, Speedy<ChunkData>>(&ro_tx, Some(LMDB_TABLE_CHUNK_STORAGE))?.unwrap();
            
            let res = chunks.get(&ro_tx, &key)?;
            ro_tx.commit()?;
            Ok(res)
        })
        .await
        .unwrap()
    }

    /// Insert a chunk into storage.
    async fn insert_chunk(&self, x: i32, z: i32, dimension: &'static str, chunk: ChunkData) -> Result<(),Self::Error> {
        let db = self.env.clone();
        
        Self::spawn_write(db.clone(), move || {
            let key = calculate_chunk_key(x,z,dimension);
            let mut rw_tx = db.write_txn()?;
            let chunks = db.open_database::<U64<LE>, Speedy<ChunkData>>(&rw_tx, Some(LMDB_TABLE_CHUNK_STORAGE))?.unwrap();
            
            chunks.put(&mut rw_tx, &key, &chunk)?;
            rw_tx.commit()?;
            Ok(())
        })
        .await
        .unwrap()
    }
}

/// Calculate key of the chunk based on its location and dimension
fn calculate_chunk_key(x: i32, z: i32, dimension: &str) -> u64 {
    let mut hasher = Xxh3::default();
    hasher.update(&x.to_le_bytes());
    hasher.update(&z.to_le_bytes());
    hasher.update(dimension.as_bytes());
    hasher.digest()
}
