use std::sync::Arc;

use dashmap::DashMap;
use pumpkin_core::math::vector2::Vector2;
use tokio::sync::RwLock;

use crate::chunk::ChunkData;

use super::chunk::ChunkRelativeCoordinates;

type LoadedChunks = Arc<DashMap<Vector2<i32>, Arc<RwLock<ChunkData>>>>;

pub struct LevelLightManager {
    loaded_chunks: LoadedChunks,
}

#[derive(Clone, Copy)]
pub enum ChunkDirection {
    XPos = 0,
    XNeg,
    ZPos,
    ZNeg,
}

impl From<usize> for ChunkDirection {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::XPos,
            1 => Self::XNeg,
            2 => Self::ZPos,
            3 => Self::ZNeg,
            _ => Self::XPos,
        }
    }
}

pub struct ChunkBoundaryPropagation {
    pub level: u8,
    pub direction: ChunkDirection,
    pub coordinates: ChunkRelativeCoordinates,
}

impl LevelLightManager {
    pub fn new(loaded_chunks: LoadedChunks) -> Self {
        Self { loaded_chunks }
    }

    fn surrounding_chunk_coordinates(chunk_coordinates: &Vector2<i32>) -> [Vector2<i32>; 4] {
        [
            Vector2::new(chunk_coordinates.x + 1, chunk_coordinates.z),
            Vector2::new(chunk_coordinates.x - 1, chunk_coordinates.z),
            Vector2::new(chunk_coordinates.x, chunk_coordinates.z + 1),
            Vector2::new(chunk_coordinates.x, chunk_coordinates.z - 1),
        ]
    }

    pub fn initialize_lighting(&self, chunk_coordinates: Vector2<i32>) {
        let surrounding_chunks = Self::surrounding_chunk_coordinates(&chunk_coordinates)
            .map(|coordinates| self.loaded_chunks.get(&coordinates));
    }
}
