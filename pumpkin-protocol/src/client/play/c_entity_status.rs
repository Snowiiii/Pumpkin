use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:entity_event")]
pub struct CEntityStatus {
    entity_id: i32,
    entity_status: i8,
}

impl CEntityStatus {
    pub fn new(entity_id: i32, entity_status: i8) -> Self {
        Self {
            entity_id,
            entity_status,
        }
    }
}
