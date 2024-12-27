use pumpkin_core::permission::PermissionLvl;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Op {
    pub uuid: Uuid,
    pub name: String,
    pub level: PermissionLvl,
    pub bypasses_player_limit: bool,
}

impl Op {
    pub fn new(
        uuid: Uuid,
        name: String,
        level: PermissionLvl,
        bypasses_player_limit: bool,
    ) -> Self {
        Self {
            uuid,
            name,
            level,
            bypasses_player_limit,
        }
    }
}
