use pumpkin_macros::packet;

use crate::slot::Slot;

#[derive(serde::Deserialize, Debug)]
#[packet(0x32)]
pub struct SSetCreativeSlot {
    pub slot: i16,
    pub clicked_item: Slot,
}
