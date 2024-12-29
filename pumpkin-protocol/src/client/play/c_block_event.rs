use pumpkin_core::math::position::WorldPosition;

use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:block_event")]
pub struct CBlockAction<'a> {
    location: &'a WorldPosition,
    action_id: u8,
    action_parameter: u8,
    block_type: VarInt,
}

impl<'a> CBlockAction<'a> {
    pub fn new(
        location: &'a WorldPosition,
        action_id: u8,
        action_parameter: u8,
        block_type: VarInt,
    ) -> Self {
        Self {
            location,
            action_id,
            action_parameter,
            block_type,
        }
    }
}
