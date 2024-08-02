use std::borrow::Borrow;

use serde::{Deserialize, Serialize};

pub struct GameData {
    blocks: Vec<Block>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Block {
    id: u32,
    name: String,
    hardness: f32,
    resistance: f32,
    #[serde(rename = "stackSize")]
    stack_size: u32,
    diggable: Option<bool>,
    #[serde(rename = "emitLight")]
    emit_light: Option<i32>,
    #[serde(rename = "filterLight")]
    filter_light: Option<i32>
}

impl Default for GameData {
    
    fn default() -> Self {

        let blocks_json = std::fs::read_to_string("blocks.json").expect("Couldn't read file");
        let blocks: Vec<Block> = serde_json::from_str(&blocks_json).expect("Couldn't parse JSON.");
        dbg!("loaded blocks.json.");
        Self {
            blocks
        }
    }
}

impl GameData {


    //get the block id integer from the block name
    pub fn get_block_id(&self, name: String) -> u32 {

        let block_id = self.blocks
            .iter()
            .map(|v| v)
            .filter(|block| block.name == name)
            .collect::<Vec<&Block>>()
            .first()
            .unwrap()
            .id;
        
        *block_id.borrow()
    }

}
