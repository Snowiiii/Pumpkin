use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, VarInt,
};

pub struct CPluginMessage<'a> {
    channel: &'a str,
    data: &'a [u8],
}

impl<'a> CPluginMessage<'a> {
    pub fn new(channel: &'a str, data: &'a [u8]) -> Self {
        Self { channel, data }
    }
}

impl<'a> Packet for CPluginMessage<'a> {
    const PACKET_ID: VarInt = 0x01;
}

impl<'a> ClientPacket for CPluginMessage<'a> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(self.channel);
        bytebuf.put_slice(self.data);
    }
}
