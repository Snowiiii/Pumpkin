use pumpkin_macros::server_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[server_packet("play:use_item")]
pub struct SUseItem {
    // 0 for main hand, 1 for off hand
    pub hand: VarInt,
    pub sequence: VarInt,
    pub yaw: f32,
    pub pitch: f32,
}
