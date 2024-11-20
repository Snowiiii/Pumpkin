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

impl LevelTime {
    pub fn new() -> Self {
        Self {
            world_age: 0,
            time_of_day: 0,
            rain_time: 0,
        }
    }

    pub async fn tick_time(&mut self) {
        self.world_age += 1;
        self.time_of_day += 1;
        self.rain_time += 1;
    }

    pub async fn send_time(&self, client: &Client) {
        client
            .send_packet(&CUpdateTime::new(self.world_age, self.time_of_day, true))
            .await;
    }

    pub async fn add_time(&mut self, time: i64) {
        self.time_of_day += time;
    }

    pub async fn set_time(&mut self, time: i64) {
        self.time_of_day = time;
    }

    pub async fn query_daytime(&self) -> i64 {
        return self.time_of_day;
    }

    pub async fn query_gametime(&self) -> i64 {
        return self.world_age;
    }

    pub async fn query_day(&self) -> i64 {
        return self.time_of_day % 24000;
    }
}
