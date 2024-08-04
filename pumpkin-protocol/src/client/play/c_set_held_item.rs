use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, VarInt,
};

pub struct CSetHeldItem {
    slot: i8,
}

impl CSetHeldItem {
    pub fn new(slot: i8) -> Self {
        Self { slot }
    }
}

impl Packet for CSetHeldItem {
    const PACKET_ID: VarInt = 0x53;
}

impl ClientPacket for CSetHeldItem {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_i8(self.slot);
    }
}
