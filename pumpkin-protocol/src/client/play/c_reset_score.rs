use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x44)]
pub struct CResetScore {
    entity_name: String,
    objective_name: Option<String>,
}
