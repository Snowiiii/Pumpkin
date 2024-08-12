use pumpkin_macros::packet;
use pumpkin_text::TextComponent;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x63)]
pub struct CSubtitle {
    subtitle: TextComponent,
}

impl CSubtitle {
    pub fn new(subtitle: TextComponent) -> Self {
        Self { subtitle }
    }
}
