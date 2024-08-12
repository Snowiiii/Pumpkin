use pumpkin_macros::packet;
use pumpkin_text::TextComponent;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x6C)]
pub struct CSystemChatMessge {
    content: TextComponent,
    overlay: bool,
}

impl CSystemChatMessge {
    pub fn new(content: TextComponent, overlay: bool) -> Self {
        Self { content, overlay }
    }
}
