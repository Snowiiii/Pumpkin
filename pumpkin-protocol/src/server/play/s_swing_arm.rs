use crate::VarInt;
use pumpkin_macros::packet;

#[derive(serde::Deserialize)]
#[packet(0x36)]
pub struct SSwingArm {
    pub hand: VarInt,
}
