use pumpkin_core::math::position::WorldPosition;

use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:block_destruction")]
pub struct CSetBlockDestroyStage {
    entity_id: VarInt,
    location: WorldPosition,
    destroy_stage: u8,
}

impl CSetBlockDestroyStage {
    pub fn new(entity_id: VarInt, location: WorldPosition, destroy_stage: u8) -> Self {
        Self {
            entity_id,
            location,
            destroy_stage,
        }
    }
}
