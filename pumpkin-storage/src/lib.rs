use std::fmt::Debug;

use pumpkin_world::chunk::ChunkData;

/// Chunk storage over LMDB
pub mod lmdb;

/// Chunk storage over .MCA folder
pub mod mca;

#[allow(async_fn_in_trait)]
/// A trait implementing the API for fetching and inserting chunks to storage
pub trait ChunkStorage: Sized + Send + Sync {
    
    /// Error returned by the database
    type Error: Debug; // In the future: + Into<PumpkinError>

    type OpenOptions: Clone; 
    
    /// Open storage
    async fn start(options: Self::OpenOptions) -> Result<Self, Self::Error>;
    
    /// Close storage
    async fn close(self);
    
    /// Get chunk from storage
    async fn get_chunk(&self, x: i32, z: i32, dimension: &'static str) -> Result<Option<ChunkData>, Self::Error>;
    
    /// Insert chunk into storage
    async fn insert_chunk(&self, x: i32, z: i32, dimension: &'static str, chunk: ChunkData) -> Result<(),Self::Error>;
}
