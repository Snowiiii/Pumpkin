use pumpkin_macros::server_packet;
use serde::Deserialize;

#[derive(Deserialize)]
#[server_packet("play:keep_alive")]
pub struct SKeepAlive {
    pub keep_alive_id: i64,
}
