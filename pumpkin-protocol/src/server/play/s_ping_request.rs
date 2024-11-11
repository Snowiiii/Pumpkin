use pumpkin_macros::server_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[server_packet("play:ping_request")]
pub struct SPlayPingRequest {
    pub payload: i64,
}
