use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:disconnect")]
pub struct CPlayDisconnect<'a> {
    reason: &'a TextComponent,
}

impl<'a> CPlayDisconnect<'a> {
    pub fn new(reason: &'a TextComponent) -> Self {
        Self { reason }
    }
}
