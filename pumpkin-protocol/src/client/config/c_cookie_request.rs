use pumpkin_macros::client_packet;

use crate::codec::identifier::Identifier;

#[derive(serde::Serialize)]
#[client_packet("config:cookie_request")]
/// Requests a cookie that was previously stored.
pub struct CCookieRequest<'a> {
    key: &'a Identifier,
}

impl<'a> CCookieRequest<'a> {
    pub fn new(key: &'a Identifier) -> Self {
        Self { key }
    }
}
