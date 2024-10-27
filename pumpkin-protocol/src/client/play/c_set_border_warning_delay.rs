use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::WorldBorderWarningDelay as i32)]
pub struct CSetBorderWarningDelay {
    warning_time: VarInt,
}

impl CSetBorderWarningDelay {
    pub fn new(warning_time: VarInt) -> Self {
        Self { warning_time }
    }
}
