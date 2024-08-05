use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x53)]
pub struct CSetHeldItem {
    slot: i8,
}

impl CSetHeldItem {
    pub fn new(slot: i8) -> Self {
        Self { slot }
    }
}
