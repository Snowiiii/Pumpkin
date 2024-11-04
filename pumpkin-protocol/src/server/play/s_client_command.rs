use pumpkin_macros::server_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[server_packet("play:client_command")]
pub struct SClientCommand {
    pub action_id: VarInt,
}
