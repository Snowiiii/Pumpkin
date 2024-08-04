use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, VarInt,
};

pub struct CGameEvent {
    event: u8,
    value: f32,
}

impl CGameEvent {
    pub fn new(event: u8, value: f32) -> Self {
        Self { event, value }
    }
}

impl Packet for CGameEvent {
    const PACKET_ID: VarInt = 0x22;
}

impl ClientPacket for CGameEvent {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_u8(self.event);
        bytebuf.put_f32(self.value);
    }
}
