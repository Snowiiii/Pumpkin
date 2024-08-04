use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, KnownPack, VarInt,
};

pub struct CKnownPacks<'a> {
    known_packs: &'a [KnownPack<'a>],
}

impl<'a> CKnownPacks<'a> {
    pub fn new(known_packs: &'a [KnownPack]) -> Self {
        Self { known_packs }
    }
}

impl<'a> Packet for CKnownPacks<'a> {
    const PACKET_ID: VarInt = 0x0E;
}

impl<'a> ClientPacket for CKnownPacks<'a> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_list::<KnownPack>(self.known_packs, |p, v| {
            p.put_string(v.namespace);
            p.put_string(v.id);
            p.put_string(v.version);
        });
    }
}
