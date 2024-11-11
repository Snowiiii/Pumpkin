use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Painting {
    asset_id: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
    //  title: Option<TextComponent<'static>>,
    //  #[serde(skip_serializing_if = "Option::is_none")]
    //  author: Option<TextComponent<'static>>,
    height: i32,
    width: i32,
}
