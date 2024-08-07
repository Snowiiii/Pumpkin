use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x01)]
pub struct CPingResponse {
    payload: i64, // must responde with the same as in `SPingRequest`
}

impl CPingResponse {
    pub fn new(payload: i64) -> Self {
        Self { payload }
    }
}
