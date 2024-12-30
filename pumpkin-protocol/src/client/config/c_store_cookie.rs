use crate::{codec::identifier::Identifier, VarInt};
use pumpkin_macros::client_packet;

#[derive(serde::Serialize)]
#[client_packet("config:store_cookie")]
/// Stores some arbitrary data on the client, which persists between server transfers.
/// The Notchian (vanilla) client only accepts cookies of up to 5 kiB in size.
pub struct CStoreCookie<'a> {
    key: &'a Identifier,
    payload_length: VarInt,
    payload: &'a [u8], // 5120,
}

impl<'a> CStoreCookie<'a> {
    pub fn new(key: &'a Identifier, payload: &'a [u8]) -> Self {
        Self {
            key,
            payload_length: VarInt(payload.len() as i32),
            payload,
        }
    }
}
