use num_derive::FromPrimitive;
use pumpkin_macros::packet;

use crate::{bytebuf::DeserializerError, ServerPacket, VarInt};

#[packet(0x25)]
pub struct SPlayerCommand {
    pub entitiy_id: VarInt,
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
    StartHourseJump,
    StopHourseJump,
    OpenVehicleInventory,
    StartFlyingElytra,
}

impl ServerPacket for SPlayerCommand {
    fn read(bytebuf: &mut crate::bytebuf::ByteBuffer) -> Result<Self, DeserializerError> {
        Ok(Self {
            entitiy_id: bytebuf.get_var_int(),
            action: bytebuf.get_var_int(),
            jump_boost: bytebuf.get_var_int(),
        })
    }
}
