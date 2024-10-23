use super::ClientboundPlayPackets;
use crate::VarInt;
use pumpkin_macros::client_packet;
use serde::Serialize;
#[client_packet(ClientboundPlayPackets::CollectItem as i32)]
#[derive(Serialize)]
pub struct CPickupItem {
    collected_entity_id: VarInt,
    collector_entity_id: VarInt,
    amount: VarInt,
}

impl CPickupItem {
    pub fn new_item(item_id: VarInt, player_id: VarInt, item_amount: u8) -> Self {
        Self {
            collected_entity_id: item_id,
            collector_entity_id: player_id,
            amount: (item_amount as i32).into(),
        }
    }

    pub fn new_xp(item_id: VarInt, player_id: VarInt) -> Self {
        Self {
            collected_entity_id: item_id,
            collector_entity_id: player_id,
            amount: 1.into(),
        }
    }
}
