use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, Identifier, KnownPack, VarInt,
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

#[derive(serde::Serialize)]
pub struct CConfigDisconnect<'a> {
    reason: &'a str,
}

impl<'a> Packet for CConfigDisconnect<'a> {
    const PACKET_ID: i32 = 0x02;
}

impl<'a> CConfigDisconnect<'a> {
    pub fn new(reason: &'a str) -> Self {
        Self { reason }
    }
}

#[derive(serde::Serialize)]
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

impl Packet for CFinishConfig {
    const PACKET_ID: VarInt = 0x03;
}

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
    pub data: Vec<u8>,
}

impl<'a> Packet for CRegistryData<'a> {
    const PACKET_ID: VarInt = 0x07;
}

impl<'a> ClientPacket for CRegistryData<'a> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(self.registry_id);
        bytebuf.put_list::<RegistryEntry>(self.entries, |p, v| {
            p.put_string(v.entry_id);
            p.put_bool(!v.data.is_empty());
            p.put_slice(&v.data);
        });
    }
}
