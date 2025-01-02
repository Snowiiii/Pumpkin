use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct PlayerProfile {
    pub uuid: Uuid,
    pub name: String,
}

impl PlayerProfile {
    pub fn new(uuid: Uuid, name: String) -> Self {
        Self { uuid, name }
    }
}
