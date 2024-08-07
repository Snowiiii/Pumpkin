use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use pumpkin_macros::packet;

use crate::{ServerPacket, VarInt};

#[derive(serde::Deserialize)]
#[packet(0x04)]
pub struct SChatCommand {
    pub command: String,
}
