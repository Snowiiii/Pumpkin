use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::OpenWindow as i32)]
pub struct COpenScreen<'a> {
    window_id: VarInt,
    window_type: VarInt,
    window_title: TextComponent<'a>,
}

impl<'a> COpenScreen<'a> {
    pub fn new(window_id: VarInt, window_type: VarInt, window_title: TextComponent<'a>) -> Self {
        Self {
            window_id,
            window_type,
            window_title,
        }
    }
}
