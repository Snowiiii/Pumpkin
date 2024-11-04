use pumpkin_macros::server_packet;

use crate::VarInt;

#[derive(serde::Deserialize)]
#[server_packet("play:accept_teleportation")]
pub struct SConfirmTeleport {
    pub teleport_id: VarInt,
}
