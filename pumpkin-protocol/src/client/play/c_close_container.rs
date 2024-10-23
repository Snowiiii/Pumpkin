use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::CloseWindow as i32)]
pub struct CCloseContainer {
    window_id: VarInt,
}

impl CCloseContainer {
    pub const fn new(window_id: VarInt) -> Self {
        Self { window_id }
    }
}
