use crate::slot::Slot;
use crate::VarInt;
use pumpkin_macros::packet;
use serde::Serialize;
#[derive(Serialize)]
#[packet(0x15)]
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
