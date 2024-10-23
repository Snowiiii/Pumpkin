use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::EntityStatus as i32)]
pub struct CEntityStatus {
    entity_id: i32,
    entity_status: i8,
}

impl CEntityStatus {
    pub fn new(entity_id: i32, entity_status: i8) -> Self {
        Self {
            entity_id,
            entity_status,
        }
    }
}
