use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, VarInt,
};

pub struct CSetCompression {
    threshold: VarInt,
}

impl CSetCompression {
    pub fn new(threshold: VarInt) -> Self {
        Self { threshold }
    }
}

impl Packet for CSetCompression {
    const PACKET_ID: VarInt = 0x03;
}

impl ClientPacket for CSetCompression {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_var_int(self.threshold);
    }
}
