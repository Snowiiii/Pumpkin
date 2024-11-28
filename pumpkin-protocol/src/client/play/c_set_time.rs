use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:set_time")]
pub struct CUpdateTime {
    world_age: i64,
    time_of_day: i64,
    time_of_day_increasing: bool,
}

impl CUpdateTime {
    pub fn new(world_age: i64, time_of_day: i64, time_of_day_increasing: bool) -> Self {
        Self {
            world_age,
            time_of_day,
            time_of_day_increasing,
        }
    }
}
