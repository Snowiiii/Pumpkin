use super::RegistryValue;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ChatType {
    chat: Decoration,
    narration: Decoration,
}
#[derive(Debug, Clone, Serialize)]
struct Decoration {
    translation_key: String,
    // style: Option<>
    parameters: Vec<String>,
}

pub(super) fn all() -> Vec<RegistryValue<ChatType>> {
    vec![RegistryValue {
        name: "minecraft:chat".into(),
        id: 0,
        element: ChatType {
            chat: Decoration {
                parameters: vec!["sender".into(), "content".into()],
                translation_key: "chat.type.text".into(),
            },
            narration: Decoration {
                parameters: vec!["sender".into(), "content".into()],
                translation_key: "chat.type.text.narrate".into(),
            },
        },
    }]
}
