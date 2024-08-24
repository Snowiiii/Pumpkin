use crate::slot::Slot;
use crate::VarInt;
use pumpkin_macros::packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[packet(0x0E)]
pub struct SClickContainer {
    window_id: u8,
    state_id: VarInt,
    slot: Slot,
    button: i8,
    mode: VarInt,
    length_of_array: VarInt,
    array_of_changed_slots: Vec<(i16, Slot)>,
    carried_item: Slot,
}
