use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::UpdateHealth as i32)]
pub struct CSetHealth {
    health: f32,
    food: VarInt,
    food_saturation: f32,
}

impl CSetHealth {
    pub fn new(health: f32, food: VarInt, food_saturation: f32) -> Self {
        Self {
            health,
            food,
            food_saturation,
        }
    }
}
