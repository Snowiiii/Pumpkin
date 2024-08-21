use pumpkin_macros::packet;
use pumpkin_core::text::TextComponent;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x65)]
pub struct CTitleText<'a> {
    title: TextComponent<'a>,
}

impl<'a> CTitleText<'a> {
    pub fn new(title: TextComponent<'a>) -> Self {
        Self { title }
    }
}
