use pumpkin_macros::client_packet;

use crate::{Identifier, VarInt};

#[derive(serde::Serialize)]
#[client_packet("config:store_cookie")]
/// Stores some arbitrary data on the client, which persists between server transfers.
/// The Notchian (vanilla) client only accepts cookies of up to 5 kiB in size.
pub struct CCookieRequest {
    key: Identifier,
    payload_length: VarInt,
    payload: Vec<u8>, // 5120,
}

impl CCookieRequest {
    #[expect(dead_code)]
    pub fn new(key: Identifier, payload_length: VarInt, payload: Vec<u8>) -> Self {
        Self {
            key,
            payload_length,
            payload,
        }
    }
}
