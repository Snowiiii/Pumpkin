use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(0x68)]
pub struct CSoundEffect<'a> {
    sound_id: VarInt,
    #[serde(skip_serializing_if = "Option::is_none")]
    sound_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_fixed_range: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    range: Option<f32>,
    sound_category: VarInt,
    effect_position_x: i32,
    effect_position_y: i32,
    effect_position_z: i32,
    volume: f32,
    pitch: f32,
    seed: f64,
}

impl<'a> CSoundEffect<'a> {
    pub fn new_from_id(
        sound_id: VarInt,
        sound_category: SoundCategory,
        effect_position_x: f64,
        effect_position_y: f64,
        effect_position_z: f64,
        volume: f32,
        pitch: f32,
        seed: f64,
    ) -> Self {
        Self {
            sound_id: VarInt(sound_id.0 + 1),
            sound_name: None,
            has_fixed_range: None,
            range: None,
            sound_category: VarInt(sound_category as i32),
            effect_position_x: (effect_position_x * 8.0) as i32,
            effect_position_y: (effect_position_y * 8.0) as i32,
            effect_position_z: (effect_position_z * 8.0) as i32,
            volume,
            pitch,
            seed,
        }
    }

    pub fn new_from_name(
        sound_name: &'a str,
        has_fixed_range: bool,
        range: Option<f32>,
        sound_category: SoundCategory,
        effect_position_x: f64,
        effect_position_y: f64,
        effect_position_z: f64,
        volume: f32,
        pitch: f32,
        seed: f64,
    ) -> Self {
        Self {
            sound_id: VarInt(0),
            sound_name: Some(sound_name),
            has_fixed_range: Some(has_fixed_range),
            range,
            sound_category: VarInt(sound_category as i32),
            effect_position_x: (effect_position_x * 8.0) as i32,
            effect_position_y: (effect_position_y * 8.0) as i32,
            effect_position_z: (effect_position_z * 8.0) as i32,
            volume,
            pitch,
            seed,
        }
    }

}

pub enum SoundCategory {
    Master,
    Music,
    Records,
    Weather,
    Blocks,
    Hostile,
    Neutral,
    Players,
    Ambient,
    Voice,
}
