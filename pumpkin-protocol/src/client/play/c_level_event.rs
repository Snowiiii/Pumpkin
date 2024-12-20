use pumpkin_core::math::position::WorldPosition;
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:level_event")]
pub struct CLevelEvent {
    event: i32,
    location: WorldPosition,
    data: i32,
    disable_relative_volume: bool,
}

impl CLevelEvent {
    pub fn new(
        event: i32,
        location: WorldPosition,
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
