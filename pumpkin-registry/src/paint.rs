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
