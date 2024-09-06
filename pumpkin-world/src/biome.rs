use serde::{Deserialize, Serialize};

// TODO make this work with the protocol
// Send by the registry
#[derive(Serialize, Deserialize, Clone, Copy)]
#[non_exhaustive]
pub enum Biome {
    Plains,
    SnowyTiga,
    // TODO list all Biomes
}
