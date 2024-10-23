use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::SystemChatMessage as i32)]
pub struct CSystemChatMessage<'a> {
    content: &'a TextComponent<'a>,
    overlay: bool,
}

impl<'a> CSystemChatMessage<'a> {
    pub fn new(content: &'a TextComponent<'a>, overlay: bool) -> Self {
        Self { content, overlay }
    }
}
