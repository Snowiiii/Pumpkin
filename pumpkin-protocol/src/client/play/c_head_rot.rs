use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x48)]
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
