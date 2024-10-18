use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x44)]
pub struct CResetScore {
    entity_name: String,
    objective_name: Option<String>,
}

impl CResetScore {
    pub fn new(entity_name: String, objective_name: Option<String>) -> Self {
        Self {
            entity_name,
            objective_name,
        }
    }
}
