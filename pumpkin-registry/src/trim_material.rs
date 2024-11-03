use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimMaterial {
    asset_name: String,
    ingredient: String,
    item_model_index: f32,
    //  description: TextComponent<'static>,
}
