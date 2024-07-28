use crate::protocol::{bytebuf::buffer::ByteBuffer, VarInt};

pub struct SStatusRequest {
    // empty
}

impl SStatusRequest {
    pub const PACKET_ID: VarInt = 0;

    pub fn read(_bytebuf: &mut ByteBuffer) -> Self {
        Self {}
    }
}

pub struct SPingRequest {
    pub payload: i64,
}

impl SPingRequest {
    pub const PACKET_ID: VarInt = 1;

    pub fn read(bytebuf: &mut ByteBuffer) -> Self {
        Self {
            payload: bytebuf.read_i64().unwrap(),
        }
    }
}
