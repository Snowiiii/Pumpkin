use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use pumpkin_macros::packet;

use crate::{ServerPacket, VarInt};

#[derive(serde::Deserialize)]
#[packet(0x1A)]
pub struct SPlayerPosition {
    pub x: f64,
    pub feet_y: f64,
    pub z: f64,
    pub ground: bool,
}
