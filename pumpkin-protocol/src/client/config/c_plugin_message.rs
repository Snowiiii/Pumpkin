use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundConfigPackets;

#[derive(Serialize)]
#[client_packet(ClientboundConfigPackets::PluginMessage as i32)]
pub struct CPluginMessage<'a> {
    channel: &'a str,
    data: &'a [u8],
}

impl<'a> CPluginMessage<'a> {
    pub fn new(channel: &'a str, data: &'a [u8]) -> Self {
        Self { channel, data }
    }
}
