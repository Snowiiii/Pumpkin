use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use dashmap::{DashMap, DashSet};
use itertools::Itertools;
use pumpkin_core::math::vector2::Vector2;
use tokio::sync::RwLock;

use crate::{chunk::ChunkData, lighting::chunk::ChunkLightData};

use super::chunk::ChunkRelativeCoordinates;

type LoadedChunks = Arc<DashMap<Vector2<i32>, Arc<RwLock<ChunkData>>>>;

pub struct LevelLightManager {
    loaded_chunks: LoadedChunks,
    modified_chunks: DashSet<Vector2<i32>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChunkDirection {
    XPos = 0,
    XNeg,
    ZPos,
    ZNeg,
}

impl ChunkDirection {
    pub fn apply(&self, chunk_coordinates: &Vector2<i32>) -> Vector2<i32> {
        match self {
            Self::XPos => Vector2::new(chunk_coordinates.x + 1, chunk_coordinates.z),
            Self::XNeg => Vector2::new(chunk_coordinates.x - 1, chunk_coordinates.z),
            Self::ZPos => Vector2::new(chunk_coordinates.x, chunk_coordinates.z + 1),
            Self::ZNeg => Vector2::new(chunk_coordinates.x, chunk_coordinates.z - 1),
        }
    }
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

#[derive(Debug)]
pub struct ChunkBoundaryPropagation {
    pub level: u8,
    pub direction: ChunkDirection,
    pub coordinates: ChunkRelativeCoordinates,
}

impl LevelLightManager {
    pub fn new(loaded_chunks: LoadedChunks) -> Self {
        Self {
            loaded_chunks,
            modified_chunks: DashSet::new(),
        }
    }

    pub fn take_modified_chunks(&mut self) -> Vec<Vector2<i32>> {
        let modified_chunks: Vec<_> = self
            .modified_chunks
            .iter()
            .map(|value| value.key().clone())
            .collect();
        self.modified_chunks.clear();

        modified_chunks
    }

    fn surrounding_chunk_coordinates(chunk_coordinates: &Vector2<i32>) -> [Vector2<i32>; 4] {
        [
            Vector2::new(chunk_coordinates.x + 1, chunk_coordinates.z),
            Vector2::new(chunk_coordinates.x - 1, chunk_coordinates.z),
            Vector2::new(chunk_coordinates.x, chunk_coordinates.z + 1),
            Vector2::new(chunk_coordinates.x, chunk_coordinates.z - 1),
        ]
    }

    pub async fn increase_light_levels(
        &self,
        chunk_coordinates: &Vector2<i32>,
        from_chunk_coordinates: &Vector2<i32>,
        increases: Vec<(ChunkRelativeCoordinates, u8)>,
    ) {
        let valid_neighbor_changes = {
            // May have deadlock if chunk is not dropped from initialize before this is called
            let Some(chunk) = self.loaded_chunks.get(chunk_coordinates) else {
                // Chunk not initalized, light will be pulled when the chunk initlializes
                return;
            };
            let mut chunk = chunk.write().await;
            // TODO: remove this clone if possible
            let blocks = chunk.blocks.clone();
            let Some(ref mut light) = chunk.light else {
                // Light not initialized, light will be pulled when the chunk's lighting initalizes
                return;
            };
            let (light_changed, neighbor_changes) =
                light.increase_light_levels(&blocks, increases).await;

            if light_changed {
                self.modified_chunks.insert(*chunk_coordinates);
            }

            neighbor_changes
                .into_iter()
                .filter(|change| {
                    change.direction.apply(chunk_coordinates) != *from_chunk_coordinates
                })
                .collect()
        };

        self.propagate_neighbor_changes(chunk_coordinates, valid_neighbor_changes)
            .await;
    }

    async fn propagate_neighbor_changes(
        &self,
        chunk_coordinates: &Vector2<i32>,
        neighbor_changes: Vec<ChunkBoundaryPropagation>,
    ) {
        let changes: HashMap<ChunkDirection, Vec<(ChunkRelativeCoordinates, u8)>> =
            neighbor_changes
                .into_iter()
                .map(|v| (v.direction, (v.coordinates, v.level)))
                .into_group_map();

        for (dir, increases) in changes {
            let to_chunk_coordinates = dir.apply(&chunk_coordinates);
            Box::pin(self.increase_light_levels(
                &to_chunk_coordinates,
                chunk_coordinates,
                increases,
            ))
            .await;
        }
    }

    pub async fn initialize_lighting(
        &self,
        chunk_coordinates: &Vector2<i32>,
        chunk: Arc<RwLock<ChunkData>>,
    ) {
        let neighbor_changes = {
            let surrounding_chunks =
                Self::surrounding_chunk_coordinates(&chunk_coordinates).map(|coordinates| {
                    self.loaded_chunks
                        .get(&coordinates)
                        .map(|chunk| chunk.clone())
                });

            let mut chunk = chunk.write().await;
            let mut light = ChunkLightData::new();

            let neighbor_changes = light.initialize(&chunk.blocks, surrounding_chunks).await;
            chunk.light = Some(light);
            neighbor_changes
        };

        self.propagate_neighbor_changes(chunk_coordinates, neighbor_changes)
            .await;
    }
}
