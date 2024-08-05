use pumpkin_macros::packet;

use crate::{bytebuf::ByteBuffer, ClientPacket};

#[packet(0x00)]
pub struct CLoginDisconnect<'a> {
    reason: &'a str,
}

impl<'a> CLoginDisconnect<'a> {
    pub fn new(reason: &'a str) -> Self {
        Self { reason }
    }
}

impl<'a> ClientPacket for CLoginDisconnect<'a> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(&serde_json::to_string_pretty(&self.reason).unwrap());
    }
}
