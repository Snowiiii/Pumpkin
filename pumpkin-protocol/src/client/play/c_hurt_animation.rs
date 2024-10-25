use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::HurtAnimation as i32)]
pub struct CHurtAnimation<'a> {
    entity_id: &'a VarInt,
    yaw: f32,
}

impl<'a> CHurtAnimation<'a> {
    pub fn new(entity_id: &'a VarInt, yaw: f32) -> Self {
        Self { entity_id, yaw }
    }
}
