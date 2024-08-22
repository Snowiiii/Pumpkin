use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(0x21)]
pub struct CUnloadChunk {
    z: i32,
    x: i32,
}

impl CUnloadChunk {
    pub fn new(x: i32, z: i32) -> Self {
        Self { z, x }
    }
}
