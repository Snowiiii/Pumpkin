use pumpkin_macros::packet;
use serde::Serialize;

use crate::text::TextComponent;

#[derive(Serialize, Clone)]
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
