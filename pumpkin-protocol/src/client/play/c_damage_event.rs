use pumpkin_core::math::vector3::Vector3;
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
    source_position: Option<Vector3<f64>>,
}

impl CDamageEvent {
    pub fn new(
        entity_id: VarInt,
        source_type_id: VarInt,
        source_cause_id: Option<VarInt>,
        source_direct_id: Option<VarInt>,
        source_position: Option<Vector3<f64>>,
    ) -> Self {
        Self {
            entity_id,
            source_type_id,
            source_cause_id: source_cause_id.map_or(VarInt(0), |id| VarInt(id.0 + 1)),
            source_direct_id: source_direct_id.map_or(VarInt(0), |id| VarInt(id.0 + 1)),
            source_position,
        }
    }
}
