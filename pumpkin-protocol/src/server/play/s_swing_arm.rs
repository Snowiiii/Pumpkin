use pumpkin_macros::server_packet;

use crate::VarInt;

#[derive(serde::Deserialize)]
#[server_packet("play:swing")]
pub struct SSwingArm {
    pub hand: VarInt,
}
