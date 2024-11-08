use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolfVariant {
    wild_texture: String,
    tame_texture: String,
    angry_texture: String,
    pub biomes: String,
}
