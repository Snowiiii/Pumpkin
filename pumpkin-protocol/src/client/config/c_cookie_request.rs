use pumpkin_macros::client_packet;

use crate::Identifier;

use super::ClientboundConfigPackets;

#[derive(serde::Serialize)]
#[client_packet(ClientboundConfigPackets::CookieRequest as i32)]
pub struct CCookieRequest {
    key: Identifier,
}
