use pumpkin_macros::packet;
use pumpkin_text::TextComponent;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x4C)]
pub struct CActionBar {
    action_bar: TextComponent,
}

impl CActionBar {
    pub fn new(action_bar: TextComponent) -> Self {
        Self { action_bar }
    }
}
