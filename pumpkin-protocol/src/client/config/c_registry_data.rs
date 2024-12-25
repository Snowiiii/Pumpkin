use bytes::{BufMut, BytesMut};
use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBufMut, codec::identifier::Identifier, ClientPacket};

#[client_packet("config:registry_data")]
pub struct CRegistryData<'a> {
    registry_id: &'a Identifier,
    entries: &'a [RegistryEntry],
}

impl<'a> CRegistryData<'a> {
    pub fn new(registry_id: &'a Identifier, entries: &'a [RegistryEntry]) -> Self {
        Self {
            registry_id,
            entries,
        }
    }
}

pub struct RegistryEntry {
    pub entry_id: Identifier,
    pub data: Option<BytesMut>,
}

impl ClientPacket for CRegistryData<'_> {
    fn write(&self, bytebuf: &mut impl BufMut) {
        bytebuf.put_identifier(self.registry_id);
        bytebuf.put_list::<RegistryEntry>(self.entries, |p, v| {
            p.put_identifier(&v.entry_id);
            p.put_option(&v.data, |p, v| p.put_slice(v));
        });
    }
}
