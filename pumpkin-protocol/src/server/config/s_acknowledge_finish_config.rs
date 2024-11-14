use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("config:finish_configuration")]
pub struct SAcknowledgeFinishConfig {}
