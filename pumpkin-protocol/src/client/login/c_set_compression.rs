use pumpkin_macros::packet;

use crate::{bytebuf::ByteBuffer, ClientPacket, VarInt};

#[packet(0x03)]
pub struct CSetCompression {
    threshold: VarInt,
}

impl CSetCompression {
    pub fn new(threshold: VarInt) -> Self {
        Self { threshold }
    }
}

impl ClientPacket for CSetCompression {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_var_int(self.threshold);
    }
}
