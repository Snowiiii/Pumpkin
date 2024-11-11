use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:block_changed_ack")]
pub struct CAcknowledgeBlockChange {
    sequence_id: VarInt,
}

impl CAcknowledgeBlockChange {
    pub fn new(sequence_id: VarInt) -> Self {
        Self { sequence_id }
    }
}
