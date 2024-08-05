use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt32;

#[derive(Serialize)]
#[packet(0x03)]
pub struct CSetCompression {
    threshold: VarInt32,
}

impl CSetCompression {
    pub fn new(threshold: VarInt32) -> Self {
        Self { threshold }
    }
}
