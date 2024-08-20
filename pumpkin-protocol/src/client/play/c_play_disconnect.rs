use pumpkin_macros::packet;
use pumpkin_text::TextComponent;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x1D)]
pub struct CPlayDisconnect<'a> {
    reason: TextComponent<'a>,
}

impl<'a> CPlayDisconnect<'a> {
    pub fn new(reason: TextComponent<'a>) -> Self {
        Self { reason }
    }
}
