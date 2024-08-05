use pumpkin_macros::packet;
use serde::Serialize;

use crate::text::TextComponent;

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
