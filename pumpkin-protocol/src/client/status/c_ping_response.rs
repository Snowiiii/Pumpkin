use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundStatusPackets;

#[derive(Serialize)]
#[client_packet(ClientboundStatusPackets::PingRequest as i32)]
pub struct CPingResponse {
    payload: i64, // must responde with the same as in `SPingRequest`
}

impl CPingResponse {
    pub fn new(payload: i64) -> Self {
        Self { payload }
    }
}
