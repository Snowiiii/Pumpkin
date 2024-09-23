use pumpkin_macros::packet;
use serde::Deserialize;

#[packet(0x18)]
#[derive(Deserialize)]
pub struct SKeepAlive {
    pub keep_alive_id: i64,
}
