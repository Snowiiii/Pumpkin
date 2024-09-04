use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x26)]
pub struct CKeepAlive {
    id: i64,
}

impl CKeepAlive {
    pub fn new(id: i64) -> Self {
        Self { id }
    }
}
