use pumpkin_macros::packet;

use crate::Identifier;

#[derive(serde::Serialize)]
#[packet(0x00)]
pub struct CCookieRequest {
    key: Identifier,
}
