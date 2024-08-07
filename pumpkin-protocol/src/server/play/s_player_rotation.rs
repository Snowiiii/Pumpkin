use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use pumpkin_macros::packet;

use crate::{ServerPacket, VarInt};

#[derive(serde::Deserialize)]
#[packet(0x1C)]
pub struct SPlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
    pub ground: bool,
}
