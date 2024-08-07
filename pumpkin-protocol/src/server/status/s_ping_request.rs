use pumpkin_macros::packet;

#[derive(serde::Deserialize)]
#[packet(0x01)]
pub struct SPingRequest {
    pub payload: i64,
}
