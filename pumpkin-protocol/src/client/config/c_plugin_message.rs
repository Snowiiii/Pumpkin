use pumpkin_macros::packet;

use crate::{bytebuf::ByteBuffer, ClientPacket};

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

impl<'a> ClientPacket for CPluginMessage<'a> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(self.channel);
        bytebuf.put_slice(self.data);
    }
}
