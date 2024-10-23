use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::CloseWindow as i32)]
pub struct CCloseContainer {
    window_id: u8,
}

impl CCloseContainer {
    pub const fn new(window_id: u8) -> Self {
        Self { window_id }
    }
}
