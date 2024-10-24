use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::SetTitleText as i32)]
pub struct CTitleText<'a> {
    title: TextComponent<'a>,
}

impl<'a> CTitleText<'a> {
    pub fn new(title: TextComponent<'a>) -> Self {
        Self { title }
    }
}
