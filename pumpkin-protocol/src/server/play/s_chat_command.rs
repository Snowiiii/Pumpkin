use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("play:chat_command")]
pub struct SChatCommand {
    pub command: String,
}
