use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundLoginPackets;

#[derive(Serialize)]
#[client_packet(ClientboundLoginPackets::Disconnect as i32)]
pub struct CLoginDisconnect<'a> {
    json_reason: &'a str,
}

impl<'a> CLoginDisconnect<'a> {
    // input json!
    pub fn new(json_reason: &'a str) -> Self {
        Self { json_reason }
    }
}
