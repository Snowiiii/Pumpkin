use pumpkin_macros::packet;

use crate::{bytebuf::ByteBuffer, ClientPacket};

#[packet(0x53)]
pub struct CSetHeldItem {
    slot: i8,
}

impl CSetHeldItem {
    pub fn new(slot: i8) -> Self {
        Self { slot }
    }
}

impl ClientPacket for CSetHeldItem {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_i8(self.slot);
    }
}
