use crate::{bytebuf::packet_id::Packet, VarInt};

#[derive(serde::Serialize)]
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

impl Packet for CFinishConfig {
    const PACKET_ID: VarInt = 0x03;
}
