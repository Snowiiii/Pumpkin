use pumpkin_macros::server_packet;

#[derive(serde::Deserialize)]
#[server_packet("play:client_tick_end")]
pub struct SClientTickEnd {}
