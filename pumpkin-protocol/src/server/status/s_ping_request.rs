use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("status:ping_request")]
pub struct SStatusPingRequest {
    pub payload: i64,
}
