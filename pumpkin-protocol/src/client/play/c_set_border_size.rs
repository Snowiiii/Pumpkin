use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::WorldBorderSize as i32)]
pub struct CSetBorderSize {
    diameter: f64,
}

impl CSetBorderSize {
    pub fn new(diameter: f64) -> Self {
        Self { diameter }
    }
}
