use pumpkin_macros::packet;
use serde::Serialize;

#[packet(0x26)]
#[derive(Serialize)]
pub struct CKeepAlive {
    keep_alive_id: i64,
}

impl CKeepAlive {
    pub fn new(keep_alive_id: i64) -> Self {
        Self { keep_alive_id }
    }
}
