use pumpkin_macros::packet;
use serde::Serialize;

use crate::position::WorldPosition;

#[derive(Serialize)]
#[packet(0x28)]
pub struct CWorldEvent {
    event: i32,
    location: WorldPosition,
    data: i32,
    disable_relative_volume: bool,
}

impl CWorldEvent {
    pub fn new(event: i32,
    location: WorldPosition,
    data: i32,
    disable_relative_volume: bool) -> Self {
        Self { event, location, data, disable_relative_volume }
    }
}