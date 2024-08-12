use pumpkin_macros::packet;
use pumpkin_text::TextComponent;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x65)]
pub struct CTitleText {
    title: TextComponent,
}

impl CTitleText {
    pub fn new(title: TextComponent) -> Self {
        Self { title }
    }
}
