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
    border_center_x: Option<f32>,
    border_center_z: Option<f32>,
    border_damage_per_block: Option<f32>,
    border_safe_zone: Option<f32>,
    border_size: Option<f32>,
    border_size_lerp_target: Option<f32>,
    border_size_lerp_time: Option<i32>,
    border_warning_blocks: Option<f32>,
    border_warning_time: Option<f32>,
    #[serde(rename = "clearWeatherTime")]
    clear_weather_time: Option<i32>,
    day_time: Option<i32>,
    difficulty: Option<i8>,
    game_type: Option<i8>,
    #[serde(rename = "hardcore")]
    hardcore: Option<i8>,
    level_name: Option<String>,
    #[serde(rename = "raining")]
    raining: Option<i8>,
    #[serde(rename = "rainTime")]
    rain_time: Option<i32>,
    #[serde(rename = "thundering")]
    thundering: Option<i8>,
    #[serde(rename = "thunderTime")]
    thunder_time: Option<i32>,
    time: Option<i32>,
    wandering_trader_spawn_chance: Option<i32>,
    wandering_trader_spawn_delay: Option<i32>,
    wandering_trader_id: Option<String>, //(adding it later)a
}

impl Default for Data {
    fn default() -> Self {
        Self {
            border_center_x: Some(0.0),
            border_center_z: Some(0.0),
            border_damage_per_block: Some(0.2),
            border_safe_zone: Some(5.0),
            border_size: Some(60000000.0),
            border_size_lerp_target: Some(60000000.0),
            border_size_lerp_time: Some(0),
            border_warning_blocks: Some(5.0),
            border_warning_time: Some(15.0),
            clear_weather_time: Some(0),
            day_time: Some(0),
            difficulty: Some(2),
            game_type: Some(0),
            hardcore: Some(0),
            level_name: Some("world".into()),
            raining: Some(0),
            rain_time: Some(0),
            thundering: Some(0),
            thunder_time: Some(0),
            time: Some(0),
            wandering_trader_spawn_chance: Some(50),
            wandering_trader_spawn_delay: Some(0),
            wandering_trader_id: Some("".into()),
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
        dbg!("loaded level.dat");
        fastnbt::from_bytes(&bytes).unwrap()
    }
    pub fn save(&self, file: &str) {
        let file = std::fs::File::open(file).unwrap();
        let new_bytes = fastnbt::to_bytes(&self).unwrap();
        let mut encoder = GzEncoder::new(file, Compression::fast());
        encoder.write_all(&new_bytes).unwrap()
    }
}
