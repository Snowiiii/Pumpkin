use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Copy)]
#[repr(u8)]
pub enum OpLevel {
    None = 0,
    Basic = 1,
    Moderator = 2,
    Admin = 3,
    Owner = 4,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Op {
    pub uuid: Uuid,
    pub name: String,
    pub level: OpLevel,
    pub bypasses_player_limit: bool,
}
