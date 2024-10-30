use pumpkin_core::math::position::WorldPosition;

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:level_event")]
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
