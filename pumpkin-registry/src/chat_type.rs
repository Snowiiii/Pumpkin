use pumpkin_core::text::style::Style;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ChatType {
    chat: Decoration,
    narration: Decoration,
}

#[derive(Debug, Clone, Serialize)]
pub struct Decoration {
    translation_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    style: Option<Style<'static>>,
    parameters: Vec<String>,
}

impl Default for ChatType {
    fn default() -> Self {
        Self {
            chat: Decoration {
                style: None,
                parameters: vec!["sender".into(), "content".into()],
                translation_key: "chat.type.text".into(),
            },
            narration: Decoration {
                style: None,
                parameters: vec!["sender".into(), "content".into()],
                translation_key: "chat.type.text.narrate".into(),
            },
        }
    }
}
