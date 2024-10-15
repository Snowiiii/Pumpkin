use crate::RegistryType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolfVariant {
    wild_texture: String,
    tame_texture: String,
    angry_texture: String,
    pub biomes: String,
}
impl RegistryType for WolfVariant {
    const REGISTRY_ID: &'static str = "minecraft:wolf_variant";
    const ENTRY_IDS: &'static [&'static str] = &[Self::REGISTRY_ID];
}
