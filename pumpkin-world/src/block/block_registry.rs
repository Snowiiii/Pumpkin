use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;

pub static BLOCKS: LazyLock<TopLevel> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../assets/blocks.json"))
        .expect("Could not parse blocks.json registry.")
});

pub static BLOCKS_BY_ID: LazyLock<HashMap<u16, Block>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for block in &BLOCKS.blocks {
        map.insert(block.id, block.clone());
    }
    map
});

pub static BLOCK_ID_BY_REGISTRY_ID: LazyLock<HashMap<String, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for block in &BLOCKS.blocks {
        map.insert(block.name.clone(), block.id);
    }
    map
});

pub static BLOCK_ID_TO_REGISTRY_ID: LazyLock<HashMap<u16, String>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for block in &*BLOCKS.blocks {
        map.insert(block.default_state_id, block.name.clone());
    }
    map
});

pub static BLOCK_ID_BY_STATE_ID: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for block in &BLOCKS.blocks {
        for state in &block.states {
            map.insert(state.id, block.id);
        }
    }
    map
});

pub static STATE_INDEX_BY_STATE_ID: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for block in &BLOCKS.blocks {
        for (index, state) in block.states.iter().enumerate() {
            map.insert(state.id, index as u16);
        }
    }
    map
});

pub static BLOCK_ID_BY_ITEM_ID: LazyLock<HashMap<u16, u16>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for block in &BLOCKS.blocks {
        map.insert(block.item_id, block.id);
    }
    map
});

pub fn get_block(registry_id: &str) -> Option<&Block> {
    let id = BLOCK_ID_BY_REGISTRY_ID.get(&registry_id.replace("minecraft:", ""))?;
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
#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct TopLevel {
    block_entity_types: Vec<String>,
    shapes: Vec<Shape>,
    pub blocks: Vec<Block>,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Block {
    pub id: u16,
    pub item_id: u16,
    pub hardness: f32,
    pub wall_variant_id: Option<u16>,
    pub translation_key: String,
    pub name: String,
    pub properties: Vec<Property>,
    pub default_state_id: u16,
    pub states: Vec<State>,
}
#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct Property {
    name: String,
    values: Vec<String>,
}
#[derive(Deserialize, Clone, Debug)]
pub struct State {
    pub id: u16,
    pub air: bool,
    pub luminance: u8,
    pub burnable: bool,
    pub opacity: Option<u32>,
    pub replaceable: bool,
    pub collision_shapes: Vec<u16>,
    pub block_entity_type: Option<u32>,
}
#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
struct Shape {
    min: [f32; 3],
    max: [f32; 3],
}
