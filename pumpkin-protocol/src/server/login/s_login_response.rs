use pumpkin_macros::packet;

// Acknowledgement to the Login Success packet sent to the server.
#[derive(serde::Deserialize)]
#[packet(0x03)]
pub struct SLoginAcknowledged {}
