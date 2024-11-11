use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:animate")]
pub struct CEntityAnimation {
    entity_id: VarInt,
    /// See `Animation`
    animation: u8,
}

impl CEntityAnimation {
    pub fn new(entity_id: VarInt, animation: u8) -> Self {
        Self {
            entity_id,
            animation,
        }
    }
}

#[repr(u8)]
pub enum Animation {
    SwingMainArm,
    LeaveBed,
    SwingOffhand,
    CriticalEffect,
    MagicCriticaleffect,
}
