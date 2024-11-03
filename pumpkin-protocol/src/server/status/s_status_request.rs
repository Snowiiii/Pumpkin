use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("status:status_request")]
pub struct SStatusRequest {
    // empty
}
