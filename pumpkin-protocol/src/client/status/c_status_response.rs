use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundStatusPackets;

#[derive(Serialize)]
#[client_packet(ClientboundStatusPackets::StatusResponse as i32)]
pub struct CStatusResponse<'a> {
    json_response: &'a str, // 32767
}
impl<'a> CStatusResponse<'a> {
    pub fn new(json_response: &'a str) -> Self {
        Self { json_response }
    }
}
