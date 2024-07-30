use crate::protocol::{registry, ClientPacket, VarInt};

pub struct CCookieRequest {
    // TODO
}

impl ClientPacket for CCookieRequest {
    const PACKET_ID: crate::protocol::VarInt = 0;

    fn write(&self, bytebuf: &mut crate::protocol::bytebuf::buffer::ByteBuffer) {}
}

pub struct CConfigDisconnect {
    reason: String,
}

impl CConfigDisconnect {
    pub fn new(reason: String) -> Self {
        Self { reason }
    }
}

impl ClientPacket for CConfigDisconnect {
    const PACKET_ID: crate::protocol::VarInt = 2;

    fn write(&self, bytebuf: &mut crate::protocol::bytebuf::buffer::ByteBuffer) {
        bytebuf.write_string(&self.reason);
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
    const PACKET_ID: crate::protocol::VarInt = 3;

    fn write(&self, _bytebuf: &mut crate::protocol::bytebuf::buffer::ByteBuffer) {}
}

pub struct CKnownPacks {
    count: VarInt,
    known_packs: Vec<KnownPack>,
}

impl CKnownPacks {
    pub fn new(count: VarInt, known_packs: Vec<KnownPack>) -> Self {
        Self { count, known_packs }
    }
}

pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

impl ClientPacket for CKnownPacks {
    const PACKET_ID: VarInt = 0x0E;

    fn write(&self, bytebuf: &mut crate::protocol::bytebuf::buffer::ByteBuffer) {
        bytebuf.write_var_int(self.count);
        bytebuf.write_list::<KnownPack>(&self.known_packs, |p, v| {
            p.write_string(&v.namespace);
            p.write_string(&v.id);
            p.write_string(&v.version);
        });
    }
}

pub struct CRegistryData {
    registry_id: String,
    entry_count: VarInt,
    entries: Vec<Entry>,
}

impl CRegistryData {
    pub fn new(registry_id: String, entry_count: VarInt, entries: Vec<Entry>) -> Self {
        Self {
            registry_id,
            entry_count,
            entries,
        }
    }
}

pub struct Entry {
    pub entry_id: String,
    pub has_data: bool,
    // data provided by registry::write_codec
}

impl ClientPacket for CRegistryData {
    const PACKET_ID: VarInt = 0x07;

    fn write(&self, bytebuf: &mut crate::protocol::bytebuf::buffer::ByteBuffer) {
        bytebuf.write_string(&self.registry_id);
        bytebuf.write_var_int(self.entry_count);
        bytebuf.write_list::<Entry>(&self.entries, |p, v| {
            p.write_string(&v.entry_id);
            p.write_bool(v.has_data);
            registry::write_codec(p, -64, 320);
        });
    }
}
