use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolfVariant {
    wild_texture: String,
    tame_texture: String,
    angry_texture: String,
    biomes: String,
}

impl Default for WolfVariant {
    fn default() -> Self {
        Self {
            wild_texture: "minecraft:entity/wolf/wolf_ashen".to_string(),
            tame_texture: "minecraft:entity/wolf/wolf_ashen_tame".to_string(),
            angry_texture: "minecraft:entity/wolf/wolf_ashen_angry".to_string(),
            biomes: "minecraft:snowy_taiga".to_string(),
        }
    }
}
