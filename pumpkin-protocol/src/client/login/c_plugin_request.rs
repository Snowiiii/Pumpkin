use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x04)]
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
