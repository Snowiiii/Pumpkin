use pumpkin_macros::packet;
use serde::Serialize;


#[derive(Serialize)]
#[packet(0x01)]
pub struct CPluginMessage<'a> {
    channel: &'a str,
    data: &'a [u8],
}

impl<'a> CPluginMessage<'a> {
    pub fn new(channel: &'a str, data: &'a [u8]) -> Self {
        Self { channel, data }
    }
}
