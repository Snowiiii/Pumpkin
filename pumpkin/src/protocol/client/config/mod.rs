use crate::protocol::{nbt::NBT, ClientPacket, VarInt};

pub struct CRegistryData {
    registry_id: String,
    entry_count: VarInt,
    entries: NBT,
}

struct Entry {
    entry_id: String,
    has_data: bool,
    data: NBT,
}

impl ClientPacket for CRegistryData {
    const PACKET_ID: VarInt = 0x07;

    fn write(&self, bytebuf: &mut crate::protocol::bytebuf::buffer::ByteBuffer) {
        bytebuf.write_string(&self.registry_id);
        bytebuf.write_var_int(self.entry_count);
    //    bytebuf.write_array(self.entries);
    }
}

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
