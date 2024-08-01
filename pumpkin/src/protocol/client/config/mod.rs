use crate::protocol::{bytebuf::ByteBuffer, ClientPacket, KnownPack, VarInt};

pub struct CCookieRequest {
    // TODO
}

impl ClientPacket for CCookieRequest {
    const PACKET_ID: crate::protocol::VarInt = 0x00;

    fn write(&self, bytebuf: &mut ByteBuffer) {}
}

pub struct CPluginMessage<'a> {
    channel: &'a str,
    data: &'a [u8],
}

impl<'a> CPluginMessage<'a> {
    pub fn new(channel: &'a str, data: &'a [u8]) -> Self {
        Self { channel, data }
    }
}

impl<'a> ClientPacket for CPluginMessage<'a> {
    const PACKET_ID: VarInt = 0x01;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(self.channel);
        bytebuf.put_slice(self.data);
    }
}

pub struct CConfigDisconnect<'a> {
    reason: &'a str,
}

impl<'a> CConfigDisconnect<'a> {
    pub fn new(reason: &'a str) -> Self {
        Self { reason }
    }
}

impl<'a> ClientPacket for CConfigDisconnect<'a> {
    const PACKET_ID: crate::protocol::VarInt = 0x02;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(self.reason);
    }
}

pub struct CFinishConfig {}

impl Default for CFinishConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl CFinishConfig {
    pub fn new() -> Self {
        Self {}
    }
}

impl ClientPacket for CFinishConfig {
    const PACKET_ID: crate::protocol::VarInt = 0x03;

    fn write(&self, _bytebuf: &mut ByteBuffer) {}
}

pub struct CKnownPacks<'a> {
    known_packs: &'a [KnownPack<'a>],
}

impl<'a> CKnownPacks<'a> {
    pub fn new(known_packs: &'a [KnownPack]) -> Self {
        Self { known_packs }
    }
}

impl<'a> ClientPacket for CKnownPacks<'a> {
    const PACKET_ID: VarInt = 0x0E;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_list::<KnownPack>(self.known_packs, |p, v| {
            p.put_string(v.namespace);
            p.put_string(v.id);
            p.put_string(v.version);
        });
    }
}

pub struct CRegistryData<'a> {
    registry_id: &'a str,
    entries: &'a [Entry<'a>],
}

impl<'a> CRegistryData<'a> {
    pub fn new(registry_id: &'a str, entries: &'a [Entry]) -> Self {
        Self {
            registry_id,
            entries,
        }
    }
}

pub struct Entry<'a> {
    pub entry_id: &'a str,
    pub has_data: bool,
    pub data: &'a [u8],
}

impl<'a> ClientPacket for CRegistryData<'a> {
    const PACKET_ID: VarInt = 0x07;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(self.registry_id);
        bytebuf.put_list::<Entry>(self.entries, |p, v| {
            p.put_string(v.entry_id);
            p.put_bool(v.has_data);
            p.put_slice(v.data);
        });
    }
}
