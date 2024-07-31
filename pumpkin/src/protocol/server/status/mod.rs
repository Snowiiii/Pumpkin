use crate::protocol::{bytebuf::ByteBuffer, VarInt};

pub struct SStatusRequest {
    // empty
}

impl SStatusRequest {
    pub const PACKET_ID: VarInt = 0x00;

    pub fn read(_bytebuf: &mut ByteBuffer) -> Self {
        Self {}
    }
}

pub struct SPingRequest {
    pub payload: i64,
}

impl SPingRequest {
    pub const PACKET_ID: VarInt = 0x01;

    pub fn read(bytebuf: &mut ByteBuffer) -> Self {
        Self {
            payload: bytebuf.get_i64(),
        }
    }
}
