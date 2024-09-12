use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x1F)]
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
