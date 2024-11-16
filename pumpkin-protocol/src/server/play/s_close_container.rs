use pumpkin_macros::server_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[server_packet("play:container_close")]
pub struct SCloseContainer {
    pub window_id: VarInt,
}
