use crate::slot::Slot;
use crate::VarInt;

use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::WindowItems as i32)]
pub struct CSetContainerContent<'a> {
    window_id: u8,
    state_id: VarInt,
    count: VarInt,
    slot_data: &'a [Slot],
    carried_item: &'a Slot,
}

impl<'a> CSetContainerContent<'a> {
    pub fn new(window_id: u8, state_id: VarInt, slots: &'a [Slot], carried_item: &'a Slot) -> Self {
        Self {
            window_id,
            state_id,
            count: slots.len().into(),
            slot_data: slots,
            carried_item,
        }
    }
}
