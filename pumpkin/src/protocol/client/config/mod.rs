use crate::protocol::ClientPacket;

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

impl CFinishConfig {
    pub fn new() -> Self {
        Self {}
    }
}

impl ClientPacket for CFinishConfig {
    const PACKET_ID: crate::protocol::VarInt = 3;

    fn write(&self, _bytebuf: &mut crate::protocol::bytebuf::buffer::ByteBuffer) {}
}
