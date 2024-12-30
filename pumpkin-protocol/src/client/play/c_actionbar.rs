use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:set_action_bar_text")]
pub struct CActionBar<'a> {
    action_bar: &'a TextComponent,
}

impl<'a> CActionBar<'a> {
    pub fn new(action_bar: &'a TextComponent) -> Self {
        Self { action_bar }
    }
}
