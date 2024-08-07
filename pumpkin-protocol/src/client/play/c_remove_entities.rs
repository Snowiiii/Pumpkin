use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize, Clone)]
#[packet(0x42)]
pub struct CRemoveEntities {
    entitiy_ids: Vec<VarInt>,
}

impl CRemoveEntities {
    pub fn new(entitiy_ids: Vec<VarInt>) -> Self {
        Self { entitiy_ids }
    }
}
