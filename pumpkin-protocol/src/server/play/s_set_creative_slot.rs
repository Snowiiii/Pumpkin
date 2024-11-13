use pumpkin_macros::server_packet;

use crate::slot::Slot;

#[derive(serde::Deserialize, Debug)]
#[server_packet("play:set_creative_mode_slot")]
pub struct SSetCreativeSlot {
    pub slot: i16,
    pub clicked_item: Slot,
}
