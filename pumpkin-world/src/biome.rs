use serde::{Deserialize, Serialize};

// TODO make this work with the protocol
#[derive(Serialize, Deserialize, Clone, Copy)]
#[non_exhaustive]
pub enum Biome {
    Plains,
    // TODO list all Biomes
}
