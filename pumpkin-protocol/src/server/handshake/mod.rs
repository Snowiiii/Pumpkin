use pumpkin_macros::packet;
use serde::Deserialize;

use crate::{ConnectionState, VarInt};

#[packet(0x00)]
#[derive(Deserialize)]
pub struct SHandShake {
    pub protocol_version: VarInt,
    pub server_address: String, // 255
    pub server_port: u16,
    pub next_state: ConnectionState,
}
