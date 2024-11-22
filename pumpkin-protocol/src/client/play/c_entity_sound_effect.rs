use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::{SoundCategory, VarInt};

#[derive(Serialize)]
#[client_packet("play:sound_entity")]
pub struct CEntitySoundEffect {
    sound_event: VarInt,
    sound_category: VarInt,
    entity_id: VarInt,
    volume: f32,
    pitch: f32,
    seed: f64,
}

impl CEntitySoundEffect {
    pub fn new(
        sound_event: VarInt,
        sound_category: SoundCategory,
        entity_id: VarInt,
        volume: f32,
        pitch: f32,
        seed: f64,
    ) -> Self {
        Self {
            sound_event: VarInt(sound_event.0 + 1),
            sound_category: VarInt(sound_category as i32),
            entity_id,
            volume,
            pitch,
            seed,
        }
    }
}
