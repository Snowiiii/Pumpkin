use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use pumpkin_macros::packet;

use crate::{bytebuf::DeserializerError, ServerPacket, VarInt};

#[packet(0x16)]
pub struct SInteract {
    pub entity_id: VarInt,
    pub typ: VarInt,
    pub target_position: Option<(f32, f32, f32)>,
    pub hand: Option<VarInt>,
    pub sneaking: bool,
}

// Great job Mojang ;D
impl ServerPacket for SInteract {
    fn read(
        bytebuf: &mut crate::bytebuf::ByteBuffer,
    ) -> Result<Self, crate::bytebuf::DeserializerError> {
        let entity_id = bytebuf.get_var_int()?;
        let typ = bytebuf.get_var_int()?;
        let action = ActionType::from_i32(typ.0).ok_or(DeserializerError::Message(
            "invalid action type".to_string(),
        ))?;
        let target_position: Option<(f32, f32, f32)> = match action {
            ActionType::Interact => None,
            ActionType::Attack => None,
            ActionType::InteractAt => {
                Some((bytebuf.get_f32()?, bytebuf.get_f32()?, bytebuf.get_f32()?))
            }
        };
        let hand = match action {
            ActionType::Interact => Some(bytebuf.get_var_int()?),
            ActionType::Attack => None,
            ActionType::InteractAt => Some(bytebuf.get_var_int()?),
        };

        Ok(Self {
            entity_id,
            typ,
            target_position,
            hand,
            sneaking: bytebuf.get_bool()?,
        })
    }
}

#[derive(FromPrimitive, PartialEq, Eq)]
pub enum ActionType {
    Interact,
    Attack,
    InteractAt,
}
