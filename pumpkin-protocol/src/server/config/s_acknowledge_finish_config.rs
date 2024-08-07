use pumpkin_macros::packet;

#[derive(serde::Deserialize)]
#[packet(0x03)]
pub struct SAcknowledgeFinishConfig {}
