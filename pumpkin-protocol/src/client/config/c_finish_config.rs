use pumpkin_macros::client_packet;

use super::ClientboundConfigPackets;

#[derive(serde::Serialize)]
#[client_packet(ClientboundConfigPackets::Finish as i32)]
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
