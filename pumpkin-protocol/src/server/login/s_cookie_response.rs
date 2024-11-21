use pumpkin_macros::server_packet;
use crate::{Identifier, VarInt};

#[derive(serde::Deserialize)]
#[server_packet("login:cookie_response")]
/// Response to a Cookie Request (login) from the server.
/// The Notchian server only accepts responses of up to 5 kiB in size.
pub struct SCookieResponse {
    pub key: Identifier,
    pub has_payload: bool,
    pub payload_length: Option<VarInt>,
    pub payload: Option<[u8; 5120]>
}
