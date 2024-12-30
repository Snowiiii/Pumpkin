use crate::codec::slot::Slot;
use crate::VarInt;

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:container_set_content")]
pub struct CSetContainerContent<'a> {
    window_id: VarInt,
    state_id: VarInt,
    count: VarInt,
    slot_data: &'a [Slot],
    carried_item: &'a Slot,
}

impl<'a> CSetContainerContent<'a> {
    pub fn new(
        window_id: VarInt,
        state_id: VarInt,
        slots: &'a [Slot],
        carried_item: &'a Slot,
    ) -> Self {
        Self {
            window_id,
            state_id,
            count: slots.len().into(),
            slot_data: slots,
            carried_item,
        }
    }
}
