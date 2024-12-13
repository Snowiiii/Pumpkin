use bytes::{BufMut, BytesMut};
use pumpkin_macros::client_packet;

use crate::{bytebuf::ByteBufMut, ClientPacket};

#[client_packet("config:registry_data")]
pub struct CRegistryData<'a> {
    registry_id: &'a str,
    entries: &'a [RegistryEntry<'a>],
}

impl<'a> CRegistryData<'a> {
    pub fn new(registry_id: &'a str, entries: &'a [RegistryEntry]) -> Self {
        Self {
            registry_id,
            entries,
        }
    }
}

pub struct RegistryEntry<'a> {
    pub entry_id: &'a str,
    pub data: BytesMut,
}

impl ClientPacket for CRegistryData<'_> {
    fn write(&self, bytebuf: &mut BytesMut) {
        bytebuf.put_string(self.registry_id);
        bytebuf.put_list::<RegistryEntry>(self.entries, |p, v| {
            p.put_string(v.entry_id);
            p.put_bool(!v.data.is_empty());
            p.put_slice(&v.data);
        });
    }
}
