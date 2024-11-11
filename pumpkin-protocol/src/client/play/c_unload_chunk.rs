use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:forget_level_chunk")]
pub struct CUnloadChunk {
    z: i32,
    x: i32,
}

impl CUnloadChunk {
    pub fn new(x: i32, z: i32) -> Self {
        Self { z, x }
    }
}
