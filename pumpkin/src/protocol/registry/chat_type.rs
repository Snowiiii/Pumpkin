use super::CodecItem;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ChatType {
    chat: ChatParams,
    narration: ChatParams,
}
#[derive(Debug, Clone, Serialize)]
struct ChatParams {
    parameters: Vec<String>,
    translation_key: String,
}

pub(super) fn all() -> Vec<CodecItem<ChatType>> {
    vec![CodecItem {
        name: "minecraft:chat".into(),
        id: 0,
        element: ChatType {
            chat: ChatParams {
                parameters: vec!["sender".into(), "content".into()],
                translation_key: "chat.type.text".into(),
            },
            narration: ChatParams {
                parameters: vec!["sender".into(), "content".into()],
                translation_key: "chat.type.text.narrate".into(),
            },
        },
    }]
}
