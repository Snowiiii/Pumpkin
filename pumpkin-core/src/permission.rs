use num_derive::{FromPrimitive, ToPrimitive};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Represents the player's permission level
///
/// Permission levels determine the player's access to commands and server operations.
/// Each numeric level corresponds to a specific role:
/// - `Zero`: None
/// - `One`: Basic
/// - `Two`: Moderator
/// - `Three`: Admin
/// - `Four`: Owner
#[derive(FromPrimitive, ToPrimitive, Clone, Copy, Default, PartialEq, Eq)]
#[repr(i8)]
pub enum PermissionLvl {
    #[default]
    Zero = 0,
    Two = 2,
    Three = 3,
    Four = 4,
}

impl PartialOrd for PermissionLvl {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (*self as u8).partial_cmp(&(*other as u8))
    }
}

impl Serialize for PermissionLvl {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for PermissionLvl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(PermissionLvl::Zero),
            2 => Ok(PermissionLvl::Two),
            3 => Ok(PermissionLvl::Three),
            4 => Ok(PermissionLvl::Four),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid value for OpLevel: {}",
                value
            ))),
        }
    }
}
