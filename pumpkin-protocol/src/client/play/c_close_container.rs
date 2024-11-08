use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:container_close")]
pub struct CCloseContainer {
    window_id: VarInt,
}

impl CCloseContainer {
    pub const fn new(window_id: VarInt) -> Self {
        Self { window_id }
    }
}
