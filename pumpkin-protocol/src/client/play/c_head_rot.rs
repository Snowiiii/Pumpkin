use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::EntityHeadLook as i32)]
pub struct CHeadRot {
    entity_id: VarInt,
    head_yaw: u8,
}

impl CHeadRot {
    pub fn new(entity_id: VarInt, head_yaw: u8) -> Self {
        Self {
            entity_id,
            head_yaw,
        }
    }
}
