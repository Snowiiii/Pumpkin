use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::AcknowledgeBlockChanges as i32)]
pub struct CAcknowledgeBlockChange {
    sequence_id: VarInt,
}

impl CAcknowledgeBlockChange {
    pub fn new(sequence_id: VarInt) -> Self {
        Self { sequence_id }
    }
}
