use crate::Identifier;
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("login:cookie_request")]
/// Requests a cookie that was previously stored.
pub struct CCookieRequest<'a> {
    key: &'a Identifier,
}

impl<'a> CCookieRequest<'a> {
    pub fn new(key: &'a Identifier) -> Self {
        Self { key }
    }
}
