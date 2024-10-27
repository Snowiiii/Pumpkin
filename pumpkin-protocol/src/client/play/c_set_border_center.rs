use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::WorldBorderCenter as i32)]
pub struct CSetBorderCenter {
    x: f64,
    z: f64,
}

impl CSetBorderCenter {
    pub fn new(x: f64, z: f64) -> Self {
        Self { x, z }
    }
}
