use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x00)]
pub struct CLoginDisconnect<'a> {
    json_reason: &'a str,
}

impl<'a> CLoginDisconnect<'a> {
    // input json!
    pub fn new(json_reason: &'a str) -> Self {
        Self { json_reason }
    }
}
