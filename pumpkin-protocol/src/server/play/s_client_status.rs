use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
pub struct SClientStatus {
    pub action_id: VarInt,
}
