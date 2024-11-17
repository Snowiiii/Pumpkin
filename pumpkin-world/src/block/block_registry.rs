use std::sync::LazyLock;

use serde::Deserialize;

pub static BLOCKS: LazyLock<TopLevel> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../assets/blocks.json"))
        .expect("Could not parse blocks.json registry.")
});

pub fn get_block(registry_id: &str) -> Option<&Block> {
    BLOCKS
        .blocks
        .iter()
        .find(|&block| block.name == registry_id)
}

pub fn get_block_by_id<'a>(id: u16) -> Option<&'a Block> {
    BLOCKS.blocks.iter().find(|&block| block.id == id)
}

pub fn get_state_by_state_id<'a>(id: u16) -> Option<&'a State> {
    get_block_and_state_by_state_id(id).map(|(_, state)| state)
}

pub fn get_block_by_state_id<'a>(id: u16) -> Option<&'a Block> {
    get_block_and_state_by_state_id(id).map(|(block, _)| block)
}

pub fn get_block_and_state_by_state_id<'a>(id: u16) -> Option<(&'a Block, &'a State)> {
    for block in &BLOCKS.blocks {
        for state in &block.states {
            if state.id == id {
                return Some((block, state));
            }
        }
    }

    None
}

pub fn get_block_by_item<'a>(item_id: u16) -> Option<&'a Block> {
    BLOCKS.blocks.iter().find(|&block| block.item_id == item_id)
}

#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct TopLevel {
    pub blocks: Vec<Block>,
    shapes: Vec<Shape>,
    pub block_entity_types: Vec<BlockEntityKind>,
}
#[derive(Deserialize, Clone, Debug)]
pub struct Block {
    pub id: u16,
    pub item_id: u16,
    pub wall_variant_id: Option<u16>,
    pub translation_key: String,
    pub name: String,
    pub properties: Vec<Property>,
    pub default_state_id: u16,
    pub states: Vec<State>,
}
#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct BlockEntityKind {
    pub id: u32,
    pub ident: String,
    name: String,
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
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
}
