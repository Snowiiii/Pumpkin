use crate::protocol::{bytebuf::ByteBuffer, ClientPacket, VarInt};

pub struct CPingResponse {
    payload: i64, // must responde with the same as in `SPingRequest`
}

impl CPingResponse {
    pub fn new(payload: i64) -> Self {
        Self { payload }
    }
}

impl ClientPacket for CPingResponse {
    const PACKET_ID: VarInt = 0x01;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_i64(self.payload);
    }
}

pub struct CStatusResponse<'a> {
    json_response: &'a str, // 32767
}

impl<'a> CStatusResponse<'a> {
    pub fn new(json_response: &'a str) -> Self {
        Self { json_response }
    }
}

impl<'a> ClientPacket for CStatusResponse<'a> {
    const PACKET_ID: VarInt = 0x00;

    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(self.json_response);
    }
}
