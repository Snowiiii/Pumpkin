use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use pumpkin_core::math::position::WorldPosition;
use pumpkin_world::block::{block_registry::{Block, BLOCKS}, BlockFace};

use crate::world::World;

use super::properties::slab::SlabBehavior;

#[async_trait]
pub trait BlockBehavior: Send + Sync {
    async fn map_state_id(&self, world: &World, block: &Block, face: &BlockFace, world_pos: &WorldPosition) -> u16;
    async fn is_updateable(&self, world: &World, block: &Block, face: &BlockFace, world_pos: &WorldPosition) -> bool;
}

#[derive(Clone, Debug)]
pub enum BlockProperty {
    Waterlogged(bool),
    Facing(Direction),
    SlabType(SlabPosition),
    // Add other properties as needed
}

#[derive(Clone, Debug)]
pub enum SlabPosition {
    Top,
    Bottom,
    Double,
}

#[derive(Clone, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

pub fn get_property_key(property_name: &str) -> Option<BlockProperty> {
    match property_name {
        "waterlogged" => Some(BlockProperty::Waterlogged(false)),
        "facing" => Some(BlockProperty::Facing(Direction::North)),
        "type" => Some(BlockProperty::SlabType(SlabPosition::Top)),
        _ => None,
    }
}

#[derive(Default)]
pub struct BlockPropertiesManager {
    properties_registry: HashMap<u16, Arc<dyn BlockBehavior>>,
}

impl BlockPropertiesManager {
    pub fn build_properties_registry(&mut self) {
        for block in BLOCKS.blocks.iter() {
            let behaviour: Arc<dyn BlockBehavior> = match block.name.as_str() {
                name if name.ends_with("_slab") => SlabBehavior::get_or_init(&block.properties),
                _ => continue,
            };
            self.properties_registry.insert(block.id, behaviour);
        }
    }

    pub async fn get_state_id(&self, world: &World, block: &Block, face: &BlockFace, world_pos: &WorldPosition) -> u16 {
        if let Some(behaviour) = self.properties_registry.get(&block.id) {
            return behaviour.map_state_id(world, block, face, world_pos).await;
        }
        block.default_state_id
    }

    pub async fn is_updateable(&self, world: &World, block: &Block, face: &BlockFace, world_pos: &WorldPosition) -> bool {
        if let Some(behaviour) = self.properties_registry.get(&block.id) {
            return behaviour.is_updateable(world, block, face, world_pos).await;
        }
        false
    }
}
