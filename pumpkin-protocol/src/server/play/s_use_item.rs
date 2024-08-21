use num_derive::FromPrimitive;
use pumpkin_macros::packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[packet(0x39)]
pub struct SUseItem {
    // 0 for main hand, 1 for off hand
    pub hand: VarInt,
    pub sequence: VarInt,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(FromPrimitive)]
pub enum Hand {
    MainHand = 0,
    OffHand,
}
