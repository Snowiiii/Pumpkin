use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::codec::identifier::Identifier;

#[derive(Serialize)]
#[client_packet("login:cookie_request")]
/// Requests a cookie that was previously stored.
pub struct CLoginCookieRequest<'a> {
    key: &'a Identifier,
}

impl<'a> CLoginCookieRequest<'a> {
    pub fn new(key: &'a Identifier) -> Self {
        Self { key }
    }
}
