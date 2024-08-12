use pumpkin_macros::packet;

use crate::VarInt;

#[derive(serde::Serialize)]
#[packet(0x54)]
pub struct CCenterChunk {
    pub chunk_x: VarInt,
    pub chunk_z: VarInt,
}
