use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x00)]
pub struct CStatusResponse<'a> {
    json_response: &'a str, // 32767
}

impl<'a> CStatusResponse<'a> {
    pub fn new(json_response: &'a str) -> Self {
        Self { json_response }
    }
}
