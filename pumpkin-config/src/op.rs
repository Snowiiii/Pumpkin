use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum OpLevel {
    None = 0,
    Basic = 1,
    Moderator = 2,
    Admin = 3,
    Owner = 4,
}

impl Serialize for OpLevel {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for OpLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(OpLevel::None),
            1 => Ok(OpLevel::Basic),
            2 => Ok(OpLevel::Moderator),
            3 => Ok(OpLevel::Admin),
            4 => Ok(OpLevel::Owner),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid value for OpLevel: {}",
                value
            ))),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Op {
    pub uuid: Uuid,
    pub name: String,
    pub level: OpLevel,
    pub bypasses_player_limit: bool,
}
