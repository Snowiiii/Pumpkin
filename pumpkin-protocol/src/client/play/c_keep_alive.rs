use pumpkin_macros::packet;
use serde::Serialize;

#[packet(0x26)]
#[derive(Serialize)]
pub struct CKeepAlive {
    pub keep_alive_id: i64,
}
