use std::collections::HashMap;
use std::sync::LazyLock;

use pumpkin_core::registries::blocks::{Block, State};
use pumpkin_data::block::BLOCKS;
use pumpkin_data::block_state::BLOCK_STATES;
use pumpkin_data::block_state_collision_shapes::BLOCK_STATE_COLLISION_SHAPES;

static BLOCKS_HASH_MAP: LazyLock<HashMap<&str, usize>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for (i, block) in BLOCKS.iter().enumerate() {
        map.insert(block.name, i);
    }
    map
});

pub fn get_block(registry_id: &str) -> Option<&Block> {
    let idx = BLOCKS_HASH_MAP.get(registry_id)?;
    BLOCKS.get(*idx)
}

pub fn get_block_by_id<'a>(id: u16) -> Option<&'a Block> {
    BLOCKS.get(id as usize)
}

pub fn get_state_by_state_id<'a>(id: u16) -> Option<&'a State> {
    BLOCK_STATES.get(id as usize)
}

pub fn get_block_by_state_id<'a>(id: u16) -> Option<&'a Block> {
    let state = get_state_by_state_id(id)?;
    get_block_by_id(state.block_id)
}

pub fn get_block_and_state_by_state_id<'a>(id: u16) -> Option<(&'a Block, &'a State)> {
    let state = get_state_by_state_id(id)?;
    Some((get_block_by_id(state.block_id)?, state))
}

/// todo: make O(1), ideally by adding an optional block_id field to Item (if that's even possible)
pub fn get_block_by_item<'a>(item_id: u16) -> Option<&'a Block> {
    BLOCKS.iter().find(|b| b.item_id == item_id)
}

// todo: consider moving this into State
pub fn get_block_state_collision_shape_ids(id: u16) -> Option<&'static [u16]> {
    BLOCK_STATE_COLLISION_SHAPES.get(id as usize).copied()
}
