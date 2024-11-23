use pumpkin_core::registries::blocks::{Block, State};
use pumpkin_macros::include_blocks;

include_blocks! {}

/// todo: make O(1)
pub fn get_block(registry_id: &str) -> Option<&Block> {
    BLOCKS.iter().find(|b| b.name == registry_id)
}

pub fn get_block_by_id<'a>(id: u16) -> Option<&'a Block> {
    BLOCKS.get(id as usize)
}

pub fn get_state_by_state_id<'a>(id: u16) -> Option<&'a State> {
    BLOCK_STATES.get(id as usize)
}

pub fn get_block_by_state_id<'a>(id: u16) -> Option<&'a Block> {
    let state = get_state_by_state_id(id)?;
    Some(get_block_by_id(state.block_id)?)
}

pub fn get_block_and_state_by_state_id<'a>(id: u16) -> Option<(&'a Block, &'a State)> {
    let state = get_state_by_state_id(id)?;
    Some((get_block_by_id(state.block_id)?, state))
}

/// todo: make O(1)
pub fn get_block_by_item<'a>(item_id: u16) -> Option<&'a Block> {
    BLOCKS.iter().find(|b| b.item_id == item_id)
}
