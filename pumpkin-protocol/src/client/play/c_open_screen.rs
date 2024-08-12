use pumpkin_macros::packet;
use pumpkin_text::TextComponent;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x33)]
pub struct COpenScreen {
    window_id: VarInt,
    window_type: VarInt,
    window_title: TextComponent,
}

impl COpenScreen {
    pub fn new(window_id: VarInt, window_type: VarInt, window_title: TextComponent) -> Self {
        Self {
            window_id,
            window_type,
            window_title,
        }
    }
}
