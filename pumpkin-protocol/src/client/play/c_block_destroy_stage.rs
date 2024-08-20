use pumpkin_macros::packet;
use serde::Serialize;

use crate::{position::WorldPosition, VarInt};

#[derive(Serialize)]
#[packet(0x06)]
pub struct CSetBlockDestroyStage {
    entity_id: VarInt,
    location: WorldPosition,
    destroy_stage: u8,
}

impl CSetBlockDestroyStage {
    pub fn new(entity_id: VarInt,
    location: WorldPosition,
    destroy_stage: u8) -> Self {
        Self { entity_id, location, destroy_stage }
    }
}