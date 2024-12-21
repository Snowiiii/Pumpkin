use pumpkin_macros::server_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[server_packet("play:pick_item_from_block")]
pub struct SPickItem {
    pub slot: VarInt,
}
