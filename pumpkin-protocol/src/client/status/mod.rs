use crate::{
    bytebuf::{packet_id::Packet, ByteBuffer},
    ClientPacket, VarInt,
};

#[derive(serde::Serialize)]
pub struct CPingResponse {
    payload: i64, // must responde with the same as in `SPingRequest`
}

impl Packet for CPingResponse {
    const PACKET_ID: VarInt = 0x01;
}

impl CPingResponse {
    pub fn new(payload: i64) -> Self {
        Self { payload }
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
impl<'a> Packet for CStatusResponse<'a> {
    const PACKET_ID: VarInt = 0x00;
}

impl<'a> ClientPacket for CStatusResponse<'a> {
    fn write(&self, bytebuf: &mut ByteBuffer) {
        bytebuf.put_string(self.json_response);
    }
}
