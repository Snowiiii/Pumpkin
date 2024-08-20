use pumpkin_macros::packet;

#[derive(serde::Deserialize)]
#[packet(0x01)]
pub struct SStatusPingRequest {
    pub payload: i64,
}
