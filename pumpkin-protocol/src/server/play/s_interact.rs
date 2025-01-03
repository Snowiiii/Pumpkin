use bytes::Buf;
use pumpkin_core::math::vector3::Vector3;
use pumpkin_macros::server_packet;

use crate::{
    bytebuf::{ByteBuf, ReadingError},
    ServerPacket, VarInt,
};

#[server_packet("play:interact")]
pub struct SInteract {
    pub entity_id: VarInt,
    pub typ: VarInt,
    pub target_position: Option<Vector3<f32>>,
    pub hand: Option<VarInt>,
    pub sneaking: bool,
}

// Great job Mojang ;D
impl ServerPacket for SInteract {
    fn read(bytebuf: &mut impl Buf) -> Result<Self, ReadingError> {
        let entity_id = bytebuf.try_get_var_int()?;
        let typ = bytebuf.try_get_var_int()?;
        let action = ActionType::try_from(typ.0)
            .map_err(|_| ReadingError::Message("invalid action type".to_string()))?;
        let target_position: Option<Vector3<f32>> = match action {
            ActionType::Interact => None,
            ActionType::Attack => None,
            ActionType::InteractAt => Some(Vector3::new(
                bytebuf.try_get_f32()?,
                bytebuf.try_get_f32()?,
                bytebuf.try_get_f32()?,
            )),
        };
        let hand = match action {
            ActionType::Interact => Some(bytebuf.try_get_var_int()?),
            ActionType::Attack => None,
            ActionType::InteractAt => Some(bytebuf.try_get_var_int()?),
        };

        Ok(Self {
            entity_id,
            typ,
            target_position,
            hand,
            sneaking: bytebuf.try_get_bool()?,
        })
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum ActionType {
    Interact,
    Attack,
    InteractAt,
}

pub struct InvalidActionType;

impl TryFrom<i32> for ActionType {
    type Error = InvalidActionType;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Interact),
            1 => Ok(Self::Attack),
            2 => Ok(Self::InteractAt),
            _ => Err(InvalidActionType),
        }
    }
}
