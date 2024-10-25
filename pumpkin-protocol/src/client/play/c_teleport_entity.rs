use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::EntityTeleport as i32)]
pub struct CTeleportEntitiy {
    entity_id: VarInt,
    x: f64,
    y: f64,
    z: f64,
    yaw: u8,
    pitch: u8,
    on_ground: bool,
}

impl CTeleportEntitiy {
    pub fn new(
        entity_id: VarInt,
        x: f64,
        y: f64,
        z: f64,
        yaw: u8,
        pitch: u8,
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            x,
            y,
            z,
            yaw,
            pitch,
            on_ground,
        }
    }
}
