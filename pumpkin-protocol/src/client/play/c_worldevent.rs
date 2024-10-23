use pumpkin_core::math::position::WorldPosition;

use pumpkin_macros::client_packet;
use serde::Serialize;

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::Effect as i32)]
pub struct CWorldEvent<'a> {
    event: i32,
    location: &'a WorldPosition,
    data: i32,
    disable_relative_volume: bool,
}

impl<'a> CWorldEvent<'a> {
    pub fn new(
        event: i32,
        location: &'a WorldPosition,
        data: i32,
        disable_relative_volume: bool,
    ) -> Self {
        Self {
            event,
            location,
            data,
            disable_relative_volume,
        }
    }
}
