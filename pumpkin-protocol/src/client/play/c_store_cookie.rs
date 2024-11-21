use crate::{Identifier, VarInt};
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:store_cookie")]
/// Stores some arbitrary data on the client, which persists between server transfers.
/// The Notchian client only accepts cookies of up to 5 kiB in size.
pub struct CStoreCookie<'a> {
    key: &'a Identifier,
    payload_length: &'a VarInt,
    payload: &'a [u8; 5120]
}

impl<'a> CStoreCookie<'a> {
    pub fn new(key: &'a Identifier, payload_length: &'a VarInt, payload: &'a [u8; 5120]) -> Self {
        Self {
            key,
            payload_length,
            payload,
        }
    }
}
