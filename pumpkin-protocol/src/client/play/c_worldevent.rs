use pumpkin_macros::packet;
use serde::Serialize;

use crate::position::WorldPosition;

#[derive(Serialize)]
#[packet(0x28)]
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
