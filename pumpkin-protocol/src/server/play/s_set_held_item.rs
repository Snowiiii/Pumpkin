use pumpkin_macros::server_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[server_packet("play:set_carried_item")]
pub struct SSetHeldItem {
    pub slot: i16,
}
