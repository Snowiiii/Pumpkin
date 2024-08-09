use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize, Clone)]
#[packet(0x42)]
pub struct CRemoveEntities {
    count: VarInt,
    entitiy_ids: Vec<VarInt>,
}

impl CRemoveEntities {
    pub fn new(entitiy_ids: Vec<VarInt>) -> Self {
        Self { count: VarInt(entitiy_ids.len() as i32), entitiy_ids }
    }
}
