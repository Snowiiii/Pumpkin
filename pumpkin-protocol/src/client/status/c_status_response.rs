use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("status:status_response")]
pub struct CStatusResponse<'a> {
    json_response: &'a str, // 32767
}
impl<'a> CStatusResponse<'a> {
    pub fn new(json_response: &'a str) -> Self {
        Self { json_response }
    }
}
