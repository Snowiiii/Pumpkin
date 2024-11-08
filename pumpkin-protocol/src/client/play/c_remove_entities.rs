use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:remove_entities")]
pub struct CRemoveEntities<'a> {
    count: VarInt,
    entity_ids: &'a [VarInt],
}

impl<'a> CRemoveEntities<'a> {
    pub fn new(entity_ids: &'a [VarInt]) -> Self {
        Self {
            count: VarInt(entity_ids.len() as i32),
            entity_ids,
        }
    }
}
