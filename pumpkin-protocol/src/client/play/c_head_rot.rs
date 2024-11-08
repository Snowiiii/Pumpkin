use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:rotate_head")]
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
