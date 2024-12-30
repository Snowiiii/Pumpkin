use pumpkin_core::text::style::Style;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatType {
    chat: Decoration,
    narration: Decoration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decoration {
    translation_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    style: Option<Style>,
    parameters: Vec<String>,
}
