use std::io::{Read, Write};

use flate2::{
    read::{GzDecoder, GzEncoder},
    Compression,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LevelDat {
    #[serde(rename = "Data")]
    pub data: Data,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Data {
    border_center_x: f32,
    border_center_z: f32,
    border_damage_per_block: f32,
    border_safe_zone: f32,
    border_size: f32,
    border_size_lerp_target: f32,
    border_size_lerp_time: i32,
    border_warning_blocks: f32,
    border_warning_time: f32,
    #[serde(rename = "clearWeatherTime")]
    clear_weather_time: i32,
    day_time: i32,
    difficulty: i8,
    game_type: i8,
    #[serde(rename = "hardcore")]
    hardcore: i8,
    level_name: String,
    #[serde(rename = "raining")]
    raining: i8,
    #[serde(rename = "rainTime")]
    rain_time: i32,
    #[serde(rename = "thundering")]
    thundering: i8,
    #[serde(rename = "thunderTime")]
    thunder_time: i32,
    time: i32,
    wandering_trader_spawn_chance: i32,
    wandering_trader_spawn_delay: i32,
    //wandering_trader_id: Uuid             (adding it later)
}

impl Default for Data {
    fn default() -> Self {
        Self {
            border_center_x: 0.0,
            border_center_z: 0.0,
            border_damage_per_block: 0.2,
            border_safe_zone: 5.0,
            border_size: 60000000.0,
            border_size_lerp_target: 60000000.0,
            border_size_lerp_time: 0,
            border_warning_blocks: 5.0,
            border_warning_time: 15.0,
            clear_weather_time: 0,
            day_time: 0,
            difficulty: 2,
            game_type: 0,
            hardcore: 0,
            level_name: "world".to_string(),
            raining: 0,
            rain_time: 0,
            thundering: 0,
            thunder_time: 0,
            time: 0,
            wandering_trader_spawn_chance: 50,
            wandering_trader_spawn_delay: 0,
        }
    }
}

impl Default for LevelDat {
    fn default() -> Self {
        Self {
            data: Data::default(),
        }
    }
}

impl LevelDat {
    pub fn load(file: &str) -> Self {
        let file = std::fs::File::open(file).unwrap();
        let mut decoder = GzDecoder::new(file);
        let mut bytes = vec![];
        decoder.read_to_end(&mut bytes).unwrap();

        fastnbt::from_bytes(&bytes).unwrap()
    }
    pub fn save(&self, file: &str) {
        let file = std::fs::File::open(file).unwrap();
        let new_bytes = fastnbt::to_bytes(&self).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::fast());
        encoder.write_all(&new_bytes).unwrap()
    }
}
