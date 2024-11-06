use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:damage_event")]
pub struct CDamageEvent {
    entity_id: VarInt,
    source_type_id: VarInt,
    source_cause_id: VarInt,
    source_direct_id: VarInt,
    source_position: Option<(f64, f64, f64)>,
}

impl CDamageEvent {
    pub fn new(
        entity_id: VarInt,
        source_type_id: VarInt,
        source_cause_id: Option<VarInt>,
        source_direct_id: Option<VarInt>,
        source_position: Option<(f64, f64, f64)>,
    ) -> Self {
        Self {
            entity_id,
            source_type_id,
            source_cause_id: source_cause_id.map_or(VarInt::new(0), |id| id + 1),
            source_direct_id: source_direct_id.map_or(VarInt::new(0), |id| id + 1),
            source_position,
        }
    }
}
