use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x24)]
pub struct CHurtAnimation {
    entitiy_id: VarInt,
    yaw: f32,
}

impl CHurtAnimation {
    pub fn new(entitiy_id: VarInt, yaw: f32) -> Self {
        Self { entitiy_id, yaw }
    }
}
