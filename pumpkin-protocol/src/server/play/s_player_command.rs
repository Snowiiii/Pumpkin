use num_derive::FromPrimitive;

use crate::{bytebuf::DeserializerError, ServerPacket, VarInt};

pub struct SPlayerCommand {
    pub entity_id: VarInt,
    pub action: VarInt,
    pub jump_boost: VarInt,
}
#[derive(FromPrimitive)]
pub enum Action {
    StartSneaking = 0,
    StopSneaking,
    LeaveBed,
    StartSprinting,
    StopSprinting,
    StartHorseJump,
    StopHorseJump,
    OpenVehicleInventory,
    StartFlyingElytra,
}

impl ServerPacket for SPlayerCommand {
    fn read(bytebuf: &mut crate::bytebuf::ByteBuffer) -> Result<Self, DeserializerError> {
        Ok(Self {
            entity_id: bytebuf.get_var_int()?,
            action: bytebuf.get_var_int()?,
            jump_boost: bytebuf.get_var_int()?,
        })
    }
}
