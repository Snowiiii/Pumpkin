use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:set_held_slot")]
pub struct CSetHeldItem {
    slot: i8,
}

impl CSetHeldItem {
    pub fn new(slot: i8) -> Self {
        Self { slot }
    }
}
