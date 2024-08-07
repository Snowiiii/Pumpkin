use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use pumpkin_macros::packet;

use crate::{ServerPacket, VarInt};

#[derive(serde::Deserialize)]
#[packet(0x00)]
pub struct SConfirmTeleport {
    pub teleport_id: VarInt,
}
