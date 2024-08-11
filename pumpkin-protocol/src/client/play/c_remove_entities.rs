use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize, Clone)]
#[packet(0x42)]
pub struct CRemoveEntities<'a> {
    count: VarInt,
    entitiy_ids: &'a [VarInt],
}

impl<'a> CRemoveEntities<'a> {
    pub fn new(entitiy_ids: &'a [VarInt]) -> Self {
        Self {
            count: VarInt(entitiy_ids.len() as i32),
            entitiy_ids,
        }
    }
}
