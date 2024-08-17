use pumpkin_macros::packet;

use crate::slot::Slot;

#[derive(serde::Deserialize, Debug)]
#[packet(0x32)]
pub struct SSetCreativeSlot {
    slot: i16,
    clicked_item: Slot,
}
