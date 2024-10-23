use crate::slot::Slot;
use crate::VarInt;

use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundPlayPackets;
#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::SetSlot as i32)]
pub struct CSetContainerSlot<'a> {
    window_id: i8,
    state_id: VarInt,
    slot: i16,
    slot_data: &'a Slot,
}

impl<'a> CSetContainerSlot<'a> {
    pub fn new(window_id: i8, state_id: i32, slot: usize, slot_data: &'a Slot) -> Self {
        Self {
            window_id,
            state_id: state_id.into(),
            slot: slot.try_into().unwrap(),
            slot_data,
        }
    }
}
