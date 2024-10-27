use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::WorldBorderWarningReach as i32)]
pub struct CSetBorderWarningDistance {
    warning_blocks: VarInt,
}

impl CSetBorderWarningDistance {
    pub fn new(warning_blocks: VarInt) -> Self {
        Self { warning_blocks }
    }
}
