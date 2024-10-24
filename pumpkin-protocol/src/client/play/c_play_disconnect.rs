use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::Disconnect as i32)]
pub struct CPlayDisconnect<'a> {
    reason: &'a TextComponent<'a>,
}

impl<'a> CPlayDisconnect<'a> {
    pub fn new(reason: &'a TextComponent<'a>) -> Self {
        Self { reason }
    }
}
