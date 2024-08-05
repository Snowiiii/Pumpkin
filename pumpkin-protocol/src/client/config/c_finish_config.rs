use pumpkin_macros::packet;

use crate::{bytebuf::packet_id::Packet, VarInt};

#[derive(serde::Serialize)]
#[packet(0x03)]
pub struct CFinishConfig {}

impl Default for CFinishConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl CFinishConfig {
    pub fn new() -> Self {
        Self {}
    }
}
