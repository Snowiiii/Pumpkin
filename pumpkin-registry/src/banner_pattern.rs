use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannerPattern {
    asset_id: String,
    translation_key: String,
}
