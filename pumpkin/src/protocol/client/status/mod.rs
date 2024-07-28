use crate::protocol::{bytebuf::buffer::ByteBuffer, ClientPacket, VarInt};

pub struct CPingResponse {
    payload: i64, // must responde with the same as in `SPingRequest`
}

impl CPingResponse {
    pub fn new(payload: i64) -> Self {
        Self { payload }
    }
}

impl ClientPacket for CPingResponse {
    const PACKET_ID: VarInt = 1;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.write_i64(self.payload);
    }
}

pub struct CStatusResponse {
    json_response: String, // 32767
}

impl CStatusResponse {
    pub fn new(json_response: String) -> Self {
        Self { json_response }
    }
}

impl ClientPacket for CStatusResponse {
    const PACKET_ID: VarInt = 0;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.write_string(self.json_response.as_str());
    }
}
