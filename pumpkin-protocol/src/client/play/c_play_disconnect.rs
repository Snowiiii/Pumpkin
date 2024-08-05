use pumpkin_macros::packet;

use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    text::TextComponent,
    ClientPacket, VarInt,
};

#[packet(0x1D)]
pub struct CPlayDisconnect {
    reason: TextComponent,
}

impl CPlayDisconnect {
    pub fn new(reason: TextComponent) -> Self {
        Self { reason }
    }
}

impl ClientPacket for CPlayDisconnect {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_slice(&self.reason.encode());
    }
}
