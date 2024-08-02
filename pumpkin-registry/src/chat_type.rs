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


