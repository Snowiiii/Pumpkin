use bytes::BytesMut;
use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBufMut, ClientPacket, KnownPack};

#[client_packet("config:select_known_packs")]
pub struct CKnownPacks<'a> {
    known_packs: &'a [KnownPack<'a>],
}

impl<'a> CKnownPacks<'a> {
    pub fn new(known_packs: &'a [KnownPack]) -> Self {
        Self { known_packs }
    }
}

impl ClientPacket for CKnownPacks<'_> {
    fn write(&self, bytebuf: &mut BytesMut) {
        bytebuf.put_list::<KnownPack>(self.known_packs, |p, v| {
            p.put_string(v.namespace);
            p.put_string(v.id);
            p.put_string(v.version);
        });
    }
}
