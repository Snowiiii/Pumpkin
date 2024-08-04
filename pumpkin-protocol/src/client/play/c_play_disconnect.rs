use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    text::Text,
    ClientPacket, VarInt,
};

pub struct CPlayDisconnect {
    reason: Text,
}

impl CPlayDisconnect {
    pub fn new(reason: Text) -> Self {
        Self { reason }
    }
}

impl Packet for CPlayDisconnect {
    const PACKET_ID: VarInt = 0x1D;
}

impl ClientPacket for CPlayDisconnect {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_slice(&self.reason.encode());
    }
}
