use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("login:login_compression")]
pub struct CSetCompression {
    threshold: VarInt,
}

impl CSetCompression {
    pub fn new(threshold: VarInt) -> Self {
        Self { threshold }
    }
}
