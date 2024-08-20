use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x36)]
pub struct CPingResponse {
    payload: i64,
}

impl CPingResponse {
    pub fn new(payload: i64) -> Self {
        Self { payload }
    }
}
