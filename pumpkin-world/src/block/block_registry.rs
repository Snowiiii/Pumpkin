use std::collections::HashMap;
use std::sync::LazyLock;

use pumpkin_core::registries::blocks::{Block, BlockEntityKind, Shape, State};
use pumpkin_data::block::BLOCKS;
use pumpkin_data::block_entities::BLOCK_ENTITY_KINDS;
use pumpkin_data::block_shapes::BLOCK_SHAPES;
use pumpkin_data::block_state::BLOCK_STATES;
use pumpkin_data::block_state_collision_shapes::BLOCK_STATE_COLLISION_SHAPES;

static BLOCKS_HASH_MAP: LazyLock<HashMap<&str, usize>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (i, block) in BLOCKS.iter().enumerate() {
        map.insert(block.name, i);
    }
    map
});

/// todo: make this const if possible
pub fn get_block(registry_id: &str) -> Option<&'static Block> {
    let idx = BLOCKS_HASH_MAP.get(registry_id)?;
    BLOCKS.get(*idx)
}

pub fn get_block_by_id(id: u16) -> Option<&'static Block> {
    BLOCKS.get(id as usize)
}

pub fn get_state_by_state_id(id: u16) -> Option<&'static State> {
    BLOCK_STATES.get(id as usize)
}

pub fn get_block_by_state_id(id: u16) -> Option<&'static Block> {
    let state = get_state_by_state_id(id)?;
    get_block_by_id(state.block_id)
}

pub fn get_block_and_state_by_state_id(id: u16) -> Option<(&'static Block, &'static State)> {
    let state = get_state_by_state_id(id)?;
    Some((get_block_by_id(state.block_id)?, state))
}

pub fn get_block_entity_kind_by_id(block_entity_kind_id: u32) -> Option<&'static BlockEntityKind> {
    BLOCK_ENTITY_KINDS.get(block_entity_kind_id as usize)
}

pub fn get_collision_shape_by_id(collision_shape_id: u16) -> Option<&'static Shape> {
    BLOCK_SHAPES.get(collision_shape_id as usize)
}

pub fn get_block_state_collision_shape_ids(block_state_id: u16) -> Option<&'static [u16]> {
    BLOCK_STATE_COLLISION_SHAPES
        .get(block_state_id as usize)
        .copied()
}

pub fn iter_block_state_collision_shapes(block_state_id: u16) -> Option<Vec<&'static Shape>> {
    let ids = get_block_state_collision_shape_ids(block_state_id)?;
    let mut vec = Vec::with_capacity(ids.len());
    for id in ids {
        vec.push(get_collision_shape_by_id(*id)?);
    }
    Some(vec)
}
