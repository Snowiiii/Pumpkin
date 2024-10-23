use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::EntityPositionSync as i32)]
pub struct CUpdateEntityPos {
    entity_id: VarInt,
    delta_x: i16,
    delta_y: i16,
    delta_z: i16,
    on_ground: bool,
}

impl CUpdateEntityPos {
    pub fn new(
        entity_id: VarInt,
        delta_x: i16,
        delta_y: i16,
        delta_z: i16,
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            delta_x,
            delta_y,
            delta_z,
            on_ground,
        }
    }
}
