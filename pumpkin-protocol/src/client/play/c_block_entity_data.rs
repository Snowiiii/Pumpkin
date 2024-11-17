use pumpkin_core::math::position::WorldPosition;
use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:block_entity_data")]
pub struct CBlockEntityData {
    location: WorldPosition,
    r#type: VarInt,
    nbt_data: Vec<u8>,
}

impl CBlockEntityData {
    pub fn new(location: WorldPosition, r#type: VarInt, nbt_data: Vec<u8>) -> Self {
        Self {
            location,
            r#type,
            nbt_data,
        }
    }
}
