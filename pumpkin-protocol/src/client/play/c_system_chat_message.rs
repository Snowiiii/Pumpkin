use pumpkin_macros::packet;
use pumpkin_text::TextComponent;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x6C)]
pub struct CSystemChatMessge<'a> {
    content: TextComponent<'a>,
    overlay: bool,
}

impl<'a> CSystemChatMessge<'a> {
    pub fn new(content: TextComponent<'a>, overlay: bool) -> Self {
        Self { content, overlay }
    }
}
