use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, Identifier, VarInt,
};

pub struct CCookieRequest {
    key: Identifier,
}

impl Packet for CCookieRequest {
    const PACKET_ID: VarInt = 0x00;
}

impl ClientPacket for CCookieRequest {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(&self.key);
    }
}
