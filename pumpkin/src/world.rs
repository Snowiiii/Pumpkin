use bytes::{Bytes, BytesMut};
use uuid::Uuid;

use crate::{entity::player::{GameMode, Player}};


pub struct World {
    pub players: Vec<Player>,
}

pub struct Information {
    border_center_x: i32,
    border_center_y: i32,
    border_center_z: i32,
    border_damage_per_block: f32,
    border_safe_zone: i32,
    border_size: i32,
    border_size_lerp_target: i32,
    border_size_lerp_time: i32,
    border_warning_blocks: i32,
    border_warning_time: i32,
    data_version: i32,
    clear_weather_time: i32,
    day_time: i32,
    difficulty: i8,
    difficulty_locked: bool,
    game_type: GameMode,
    level_name: String,
    raining: bool,
    rain_time: i32,
    spawn_angle: f32,
    spawn_x: f32,
    spawn_y: f32,
    spawn_z: f32,
    thundering: bool,
    thunder_time: i32,
    wandering_trader_spawn_chance: i32,
    wandering_trader_id: Uuid,
    wandering_trader_spawn_delay: i32,
    hardcore: bool,
}

impl Information {
    
    pub fn load(file: &str) {
        
        
        
    }
    
}

pub struct Chunk {
    
    
    
}

impl World {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
        }
    }
}
