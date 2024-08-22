use pumpkin_core::text::TextComponent;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x63)]
pub struct CSubtitle<'a> {
    subtitle: TextComponent<'a>,
}

impl<'a> CSubtitle<'a> {
    pub fn new(subtitle: TextComponent<'a>) -> Self {
        Self { subtitle }
    }
}
