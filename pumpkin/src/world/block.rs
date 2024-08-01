use std::collections::HashMap;

use fastanvil::biome::Biome;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Block {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub id: i32,
    pub biome: Biome,
    pub properties: HashMap<String, String>,
}
