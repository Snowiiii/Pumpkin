use pumpkin_macros::server_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[server_packet("play:pick_item")]
pub struct SSetHeldItem {
    pub slot: i16,
}
