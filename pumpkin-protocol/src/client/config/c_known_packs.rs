use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBuffer, ClientPacket, KnownPack};

use super::ClientboundConfigPackets;

#[client_packet(ClientboundConfigPackets::KnownPacks as i32)]
pub struct CKnownPacks<'a> {
    known_packs: &'a [KnownPack<'a>],
}

impl<'a> CKnownPacks<'a> {
    pub fn new(known_packs: &'a [KnownPack]) -> Self {
        Self { known_packs }
    }
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
