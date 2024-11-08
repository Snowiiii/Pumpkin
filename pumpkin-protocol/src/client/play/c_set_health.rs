use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:set_health")]
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
