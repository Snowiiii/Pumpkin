use pumpkin_core::text::TextComponent;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x6C)]
pub struct CSystemChatMessage<'a> {
    content: &'a TextComponent<'a>,
    overlay: bool,
}

impl<'a> CSystemChatMessage<'a> {
    pub fn new(content: &'a TextComponent<'a>, overlay: bool) -> Self {
        Self { content, overlay }
    }
}
