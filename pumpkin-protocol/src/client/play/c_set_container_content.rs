use crate::slot::Slot;
use crate::VarInt;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x13)]
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
            count: slots.len().try_into().unwrap(),
            slot_data: slots,
            carried_item,
        }
    }
}
