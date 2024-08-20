use pumpkin_macros::packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[packet(0x21)]
pub struct SPlayPingRequest {
    pub payload: i64,
}
