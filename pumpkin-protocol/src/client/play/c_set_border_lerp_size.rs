use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarLong;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::WorldBorderLerpSize as i32)]
pub struct CSetBorderLerpSize {
    old_diameter: f64,
    new_diameter: f64,
    speed: VarLong,
}

impl CSetBorderLerpSize {
    pub fn new(old_diameter: f64, new_diameter: f64, speed: VarLong) -> Self {
        Self {
            old_diameter,
            new_diameter,
            speed,
        }
    }
}
