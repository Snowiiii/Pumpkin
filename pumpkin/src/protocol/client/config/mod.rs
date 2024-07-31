use crate::protocol::{bytebuf::ByteBuffer, registry, ClientPacket, KnownPack, VarInt};

pub struct CCookieRequest {
    // TODO
}

impl ClientPacket for CCookieRequest {
    const PACKET_ID: crate::protocol::VarInt = 0x00;

    fn write(&self, bytebuf: &mut ByteBuffer) {}
}

pub struct CPluginMessage<'a> {
    channel: String,
    data: &'a [u8],
}

impl<'a> CPluginMessage<'a> {
    pub fn new(channel: String, data: &'a [u8]) -> Self {
        Self { channel, data }
    }
}

impl<'a> ClientPacket for CPluginMessage<'a> {
    const PACKET_ID: VarInt = 0x01;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(&self.channel);
        bytebuf.put_slice(&self.data);
    }
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
    const PACKET_ID: crate::protocol::VarInt = 0x02;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(&self.reason);
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
    count: VarInt,
    known_packs: &'a [KnownPack],
}

impl<'a> CKnownPacks<'a> {
    pub fn new(count: VarInt, known_packs: &'a [KnownPack]) -> Self {
        Self { count, known_packs }
    }
}

impl<'a> ClientPacket for CKnownPacks<'a> {
    const PACKET_ID: VarInt = 0x0E;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        //  bytebuf.write_var_int(self.count);
        bytebuf.put_list::<KnownPack>(&self.known_packs, |p, v| {
            p.put_string(&v.namespace);
            p.put_string(&v.id);
            p.put_string(&v.version);
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

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(&self.registry_id);
        bytebuf.put_var_int(self.entry_count);
        bytebuf.put_list::<Entry>(&self.entries, |p, v| {
            p.put_string(&v.entry_id);
            p.put_bool(v.has_data);
            registry::write_single_dimension(p, -64, 320);
        });
    }
}
