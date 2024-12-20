use bytes::Bytes;
use num_derive::FromPrimitive;
use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuf, ReadingError},
    ServerPacket, VarInt,
};

#[server_packet("play:player_command")]
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
    fn read(bytebuf: &mut Bytes) -> Result<Self, ReadingError> {
        Ok(Self {
            entity_id: bytebuf.try_get_var_int()?,
            action: bytebuf.try_get_var_int()?,
            jump_boost: bytebuf.try_get_var_int()?,
        })
    }
}
