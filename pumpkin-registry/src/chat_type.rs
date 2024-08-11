use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ChatType {
    chat: Decoration,
    narration: Decoration,
}

#[derive(Debug, Clone, Serialize)]
pub struct Decoration {
    translation_key: String,
    style: u8,
    // TODO
    // style: Option<Styles>,
    parameters: Vec<String>,
}

impl Default for ChatType {
    fn default() -> Self {
        Self {
            chat: Decoration {
                style: 0,
                parameters: vec!["sender".into(), "content".into()],
                translation_key: "chat.type.text".into(),
            },
            narration: Decoration {
                style: 0,
                parameters: vec!["sender".into(), "content".into()],
                translation_key: "chat.type.text.narrate".into(),
            },
        }
    }
}
