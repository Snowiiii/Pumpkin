use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrimPattern {
    asset_id: String,
    template_item: String,
    //  description: TextComponent<'static>,
    decal: u8,
}
