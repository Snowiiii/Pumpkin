use crate::RegistryType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Painting {
    asset_id: String,
    height: i32,
    width: i32,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            asset_id: "minecraft:backyard".into(),
            height: 2,
            width: 2,
        }
    }
}
impl RegistryType for Painting {
    const REGISTRY_ID: &'static str = "minecraft:painting_variant";
    const ENTRY_IDS: &'static [&'static str] = &[Self::REGISTRY_ID];
}
