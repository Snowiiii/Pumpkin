use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("configuration:finish_configuration")]
pub struct SAcknowledgeFinishConfig {}
