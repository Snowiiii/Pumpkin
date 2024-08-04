use crate::{bytebuf::packet_id::Packet, VarInt};

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
