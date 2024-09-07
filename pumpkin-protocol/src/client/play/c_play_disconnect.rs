use pumpkin_core::text::TextComponent;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x1D)]
pub struct CPlayDisconnect<'a> {
    reason: &'a TextComponent<'a>,
}

impl<'a> CPlayDisconnect<'a> {
    pub fn new(reason: &'a TextComponent<'a>) -> Self {
        Self { reason }
    }
}
