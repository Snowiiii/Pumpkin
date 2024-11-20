use pumpkin_protocol::client::play::CUpdateTime;

use crate::client::Client;

pub struct LevelTime {
    pub world_age: i64,
    pub time_of_day: i64,
    pub rain_time: i64,
}

pub enum DayTime {
    Day = 1000,
    Night = 13000,
    Noon = 6000,
    Midnight = 18000,
}

impl Default for LevelTime {
    fn default() -> Self {
        Self::new()
    }
}

impl LevelTime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            world_age: 0,
            time_of_day: 0,
            rain_time: 0,
        }
    }

    pub fn tick_time(&mut self) {
        self.world_age += 1;
        self.time_of_day += 1;
        self.rain_time += 1;
    }

    pub async fn send_time(&self, client: &Client) {
        client
            .send_packet(&CUpdateTime::new(self.world_age, self.time_of_day, true))
            .await;
    }

    pub fn add_time(&mut self, time: i64) {
        self.time_of_day += time;
    }

    pub fn set_time(&mut self, time: i64) {
        self.time_of_day = time;
    }

    #[must_use]
    pub const fn query_daytime(&self) -> i64 {
        self.time_of_day
    }

    #[must_use]
    pub const fn query_gametime(&self) -> i64 {
        self.world_age
    }

    #[must_use]
    pub const fn query_day(&self) -> i64 {
        self.time_of_day % 24000
    }
}
