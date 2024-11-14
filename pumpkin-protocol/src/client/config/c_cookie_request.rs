use pumpkin_macros::client_packet;

use crate::Identifier;

#[derive(serde::Serialize)]
#[client_packet("config:cookie_request")]
pub struct CCookieRequest {
    key: Identifier,
}
