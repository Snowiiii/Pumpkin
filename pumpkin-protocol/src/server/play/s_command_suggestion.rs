use pumpkin_macros::server_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[server_packet("play:command_suggestion")]
pub struct SCommandSuggestion {
    pub id: VarInt,
    pub command: String,
}