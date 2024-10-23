use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundLoginPackets;

#[derive(Serialize)]
#[client_packet(ClientboundLoginPackets::LoginPluginRequest as i32)]
pub struct CLoginPluginRequest<'a> {
    message_id: VarInt,
    channel: &'a str,
    data: &'a [u8],
}

impl<'a> CLoginPluginRequest<'a> {
    pub fn new(message_id: VarInt, channel: &'a str, data: &'a [u8]) -> Self {
        Self {
            message_id,
            channel,
            data,
        }
    }
}
