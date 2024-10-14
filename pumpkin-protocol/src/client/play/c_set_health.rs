use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x5D)]
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
