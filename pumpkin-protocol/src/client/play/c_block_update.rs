use pumpkin_core::math::position::WorldPosition;

use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:block_update")]
pub struct CBlockUpdate<'a> {
    location: &'a WorldPosition,
    block_id: VarInt,
}

impl<'a> CBlockUpdate<'a> {
    pub fn new(location: &'a WorldPosition, block_id: VarInt) -> Self {
        Self { location, block_id }
    }
}
