use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:reset_score")]
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
