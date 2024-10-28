use pumpkin_macros::client_packet;
use serde::Serialize;


#[derive(Serialize)]
#[client_packet("play:set_time")]
pub struct CUpdateTime {
    world_age: u64,
    time_of_day: u64
}

impl CUpdateTime {
    pub fn new(world_age: u64, time_of_day: u64) -> Self {
        Self {
            world_age,
            time_of_day
        }
    }
}