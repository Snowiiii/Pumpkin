use pumpkin_macros::client_packet;

use crate::VarInt;

#[derive(serde::Serialize)]
#[client_packet("play:set_chunk_cache_center")]
pub struct CCenterChunk {
    pub chunk_x: VarInt,
    pub chunk_z: VarInt,
}
