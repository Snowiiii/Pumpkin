use std::{future::Future, sync::{atomic::{AtomicUsize, Ordering}, Arc, OnceLock}};

use crossbeam::sync::ShardedLock;
use futures::channel::oneshot::{self, Canceled};
use heed::{Env, Error};
use rayon::ThreadPool;

use super::{page_calculation, LMDBStorage};

pub static LMDB_READER_THREADPOOL: OnceLock<ThreadPool> = OnceLock::new();
pub static LMDB_WRITER_THREAD: OnceLock<ThreadPool> = OnceLock::new();
pub static LMDB_RESIZE_HANDLE: OnceLock<ShardedLock<()>> = OnceLock::new();
pub static LMDB_MAP_RUNTIME_SIZE: OnceLock<Arc<AtomicUsize>> = OnceLock::new();
pub static LMDB_INCREMENT_RUNTIME: OnceLock<usize> = OnceLock::new();

impl LMDBStorage {
    
    /// Spawn a database reading operations into the reading threadpool.
    pub fn spawn_read<R,F>(f: F) -> impl Future<Output = Result<R,Canceled>> 
    where 
        F: Fn() -> R + Send + 'static,
        R: Send + 'static
    {        
        let (tx,rx) = oneshot::channel::<R>();
        LMDB_READER_THREADPOOL.get().unwrap()
            .spawn(move || {
                
                // Obtain resize read lock
                let rz_lock = LMDB_RESIZE_HANDLE.get().unwrap().read().unwrap();
                
                // Execute operation
                let res = f();
                
                // Drop lock
                drop(rz_lock);
                
                // Send back result
                if tx.send(res).is_err() {
                    // LOG: warn!("Receiver closed on the other side! A database operation has been wasted.")
                }
            });
        rx
    }
    
    /// Spawn a database reading operations into the reading threadpool.
    pub fn spawn_write<R,F>(db: Env, f: F) -> impl Future<Output = Result<Result<R,heed::Error>,Canceled>> 
    where 
        F: Fn() -> Result<R,heed::Error> + Send + 'static,
        R: Send + 'static
    {        
        let (tx,rx) = oneshot::channel::<Result<R,heed::Error>>();
        LMDB_WRITER_THREAD.get().unwrap()
            .spawn(move || {
                
                
                let res = || {
                    // Obtain resize read lock
                    let rz_lock = LMDB_RESIZE_HANDLE.get().unwrap().read().unwrap();
                    
                    // Execute operation
                    let mut res = f();
                    
                    // Drop lock
                    drop(rz_lock);
                    
                    // Resizing map if full
                    while let Err(Error::Mdb(heed::MdbError::MapFull)) = res {
                        
                        // LOG: warn!("LMDB Map is full. Resizing...")
                        
                        // When the map is full we get the write lock of the resize handle. Ensuring we have exclusive access
                        // to the database. This let use the resize functionality since no transaction is ongoing.
                        let rz_lock = LMDB_RESIZE_HANDLE.get().unwrap().write().unwrap();
                        
                        let pt = LMDB_MAP_RUNTIME_SIZE.get().unwrap();
                        let old_size = pt.load(Ordering::Relaxed);
                        let new_size = page_calculation(old_size);
                        unsafe { db.resize(new_size) }?;
                        pt.store(new_size, Ordering::Relaxed);
                        
                        // LOG: info!("LMDB Map successfully resized from {} MiB to {} MiB", (old_size / 1024usize.pow(2)), (new_size / 1024usize.pow(2)))
                        
                        res = f();
                        
                        drop(rz_lock);
                    }
                    
                    res
                };
                
                
                // Send back result
                if tx.send(res()).is_err() {
                    // LOG: warn!("Receiver closed on the other side! A database operation has been wasted.")
                }
            });
        rx
    }
}
