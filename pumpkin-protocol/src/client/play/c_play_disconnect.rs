use pumpkin_macros::packet;
use pumpkin_text::TextComponent;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x1D)]
pub struct CPlayDisconnect {
    reason: TextComponent,
}

impl CPlayDisconnect {
    pub fn new(reason: TextComponent) -> Self {
        Self { reason }
    }
}
