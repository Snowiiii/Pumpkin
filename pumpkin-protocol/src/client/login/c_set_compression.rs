use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

use super::ClientboundLoginPackets;

#[derive(Serialize)]
#[client_packet(ClientboundLoginPackets::SetCompression as i32)]
pub struct CSetCompression {
    threshold: VarInt,
}

impl CSetCompression {
    pub fn new(threshold: VarInt) -> Self {
        Self { threshold }
    }
}
