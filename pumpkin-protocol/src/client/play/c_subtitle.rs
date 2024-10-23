use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::SetTitleSubtitle as i32)]
pub struct CSubtitle<'a> {
    subtitle: TextComponent<'a>,
}

impl<'a> CSubtitle<'a> {
    pub fn new(subtitle: TextComponent<'a>) -> Self {
        Self { subtitle }
    }
}
