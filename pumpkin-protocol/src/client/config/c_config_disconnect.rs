use pumpkin_macros::client_packet;

use super::ClientboundConfigPackets;

#[derive(serde::Serialize)]
#[client_packet(ClientboundConfigPackets::Disconnect as i32)]
pub struct CConfigDisconnect<'a> {
    reason: &'a str,
}

impl<'a> CConfigDisconnect<'a> {
    pub fn new(reason: &'a str) -> Self {
        Self { reason }
    }
}
