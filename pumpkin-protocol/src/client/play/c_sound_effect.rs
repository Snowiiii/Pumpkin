use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::{SoundCategory, VarInt};

#[derive(Serialize)]
#[client_packet("play:sound")]
pub struct CSoundEffect {
    sound_id: VarInt,
    // TODO: add sound from name
    // sound_name: Option<&'a str>,
    // has_fixed_range: Option<bool>,
    // range: Option<f32>,
    sound_category: VarInt,
    effect_position_x: i32,
    effect_position_y: i32,
    effect_position_z: i32,
    volume: f32,
    pitch: f32,
    seed: f64,
}

impl CSoundEffect {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
