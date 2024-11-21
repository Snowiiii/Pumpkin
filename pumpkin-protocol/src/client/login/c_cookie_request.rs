use pumpkin_macros::client_packet;
use serde::Serialize;
use crate::Identifier;

#[derive(Serialize)]
#[client_packet("login:cookie_request")]
/// Requests a cookie that was previously stored.
pub struct CCookieRequest {
    key: Identifier,
}

impl CCookieRequest {
    pub fn new(key: Identifier) -> Self {
        Self { key }
    }
}
