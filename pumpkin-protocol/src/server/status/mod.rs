use crate::{bytebuf::ByteBuffer, ServerPacket, VarInt};

pub struct SStatusRequest {
    // empty
}

impl ServerPacket for SStatusRequest {
    const PACKET_ID: VarInt = 0x00;

    fn read(_bytebuf: &mut ByteBuffer) -> Self {
        Self {}
    }
}

pub struct SPingRequest {
    pub payload: i64,
}

impl ServerPacket for SPingRequest {
    const PACKET_ID: VarInt = 0x01;

    fn read(bytebuf: &mut ByteBuffer) -> Self {
        Self {
            payload: bytebuf.get_i64(),
        }
    }
}
