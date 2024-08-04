use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, VarInt,
};

pub struct CLoginDisconnect<'a> {
    reason: &'a str,
}

impl<'a> CLoginDisconnect<'a> {
    pub fn new(reason: &'a str) -> Self {
        Self { reason }
    }
}

impl<'a> Packet for CLoginDisconnect<'a> {
    const PACKET_ID: VarInt = 0x00;
}

impl<'a> ClientPacket for CLoginDisconnect<'a> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(&serde_json::to_string_pretty(&self.reason).unwrap());
    }
}
