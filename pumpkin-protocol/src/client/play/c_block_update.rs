use pumpkin_core::math::position::WorldPosition;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x09)]
pub struct CBlockUpdate<'a> {
    location: &'a WorldPosition,
    block_id: VarInt,
}

impl<'a> CBlockUpdate<'a> {
    pub fn new(location: &'a WorldPosition, block_id: VarInt) -> Self {
        Self { location, block_id }
    }
}
