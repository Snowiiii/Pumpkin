use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x24)]
pub struct CHurtAnimation<'a> {
    entity_id: &'a VarInt,
    yaw: f32,
}

impl<'a> CHurtAnimation<'a> {
    pub fn new(entity_id: &'a VarInt, yaw: f32) -> Self {
        Self { entity_id, yaw }
    }
}
