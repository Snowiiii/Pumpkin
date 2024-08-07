use pumpkin_macros::packet;

use crate::VarInt;

#[derive(serde::Deserialize)]
#[packet(0x00)]
pub struct SConfirmTeleport {
    pub teleport_id: VarInt,
}
