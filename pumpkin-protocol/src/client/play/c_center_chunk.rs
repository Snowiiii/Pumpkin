use pumpkin_macros::client_packet;

use crate::VarInt;

use super::ClientboundPlayPackets;

#[derive(serde::Serialize)]
#[client_packet(ClientboundPlayPackets::UpdateViewPosition as i32)]
pub struct CCenterChunk {
    pub chunk_x: VarInt,
    pub chunk_z: VarInt,
}
