use pumpkin_macros::client_packet;

use crate::Identifier;

#[derive(serde::Serialize)]
#[client_packet("config:cookie_request")]
/// Requests a cookie that was previously stored.
pub struct CCookieRequest {
    key: Identifier,
}
