use std::collections::HashMap;
use std::sync::LazyLock;

use pumpkin_macros::blocks;

blocks!()

pub static BLOCKS_BY_ID: LazyLock<HashMap<u16, Block>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    //for block in &BLOCKS.blocks {
    //    map.insert(block.id, block.clone());
    //}
    map
});

pub static BLOCK_ID_BY_REGISTRY_ID: LazyLock<HashMap<String, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    //for block in &BLOCKS.blocks {
    //    map.insert(block.name.clone(), block.id);
    //}
    map
});

pub static BLOCK_ID_BY_STATE_ID: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    //for block in &BLOCKS.blocks {
    //    for state in &block.states {
    //        map.insert(state.id, block.id);
    //    }
    //}
    map
});

pub static STATE_INDEX_BY_STATE_ID: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    //for block in &BLOCKS.blocks {
    //    for (index, state) in block.states.iter().enumerate() {
    //        map.insert(state.id, index as u16);
    //    }
    //}
    map
});

pub static BLOCK_ID_BY_ITEM_ID: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    //for block in &BLOCKS.blocks {
    //    map.insert(block.item_id, block.id);
    //}
    map
});

pub fn get_block(registry_id: &str) -> Option<&Block> {
    let id = BLOCK_ID_BY_REGISTRY_ID.get(registry_id)?;
    BLOCKS_BY_ID.get(id)
}

pub fn get_block_by_id<'a>(id: u16) -> Option<&'a Block> {
    BLOCKS_BY_ID.get(&id)
}

pub fn get_state_by_state_id<'a>(id: u16) -> Option<&'a State> {
    get_block_and_state_by_state_id(id).map(|(_, state)| state)
}

pub fn get_block_by_state_id<'a>(id: u16) -> Option<&'a Block> {
    let block_id = BLOCK_ID_BY_STATE_ID.get(&id)?;
    BLOCKS_BY_ID.get(block_id)
}

pub fn get_block_and_state_by_state_id<'a>(id: u16) -> Option<(&'a Block, &'a State)> {
    let block_id = BLOCK_ID_BY_STATE_ID.get(&id)?;
    let block = BLOCKS_BY_ID.get(block_id)?;
    let state_index = STATE_INDEX_BY_STATE_ID.get(&id)?;
    let state = block.states.get(*state_index as usize)?;
    Some((block, state))
}

pub fn get_block_by_item<'a>(item_id: u16) -> Option<&'a Block> {
    let block_id = BLOCK_ID_BY_ITEM_ID.get(&item_id)?;
    BLOCKS_BY_ID.get(block_id)
}
