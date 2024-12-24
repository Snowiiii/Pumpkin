use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimMaterial {
    asset_name: String,
    ingredient: String,
    //  description: TextComponent<'static>,
}
