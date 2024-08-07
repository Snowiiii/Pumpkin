use pumpkin_macros::packet;

#[derive(serde::Deserialize)]
#[packet(0x00)]
pub struct SStatusRequest {
    // empty
}
