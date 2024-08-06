use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize, Clone)]
#[packet(0x2E)]
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
