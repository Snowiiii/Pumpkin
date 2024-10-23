use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::ActionBar as i32)]
pub struct CActionBar<'a> {
    action_bar: TextComponent<'a>,
}

impl<'a> CActionBar<'a> {
    pub fn new(action_bar: TextComponent<'a>) -> Self {
        Self { action_bar }
    }
}
