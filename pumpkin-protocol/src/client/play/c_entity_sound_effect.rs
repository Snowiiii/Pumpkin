use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::{IDOrSoundEvent, SoundCategory, SoundEvent, VarInt};

#[derive(Serialize)]
#[client_packet("play:sound_entity")]
pub struct CEntitySoundEffect {
    sound_event: IDOrSoundEvent,
    sound_category: VarInt,
    entity_id: VarInt,
    volume: f32,
    pitch: f32,
    seed: f64,
}

impl CEntitySoundEffect {
    pub fn new(
        sound_id: VarInt,
        sound_event: Option<SoundEvent>,
        sound_category: SoundCategory,
        entity_id: VarInt,
        volume: f32,
        pitch: f32,
        seed: f64,
    ) -> Self {
        Self {
            sound_event: IDOrSoundEvent {
                id: VarInt(sound_id.0 + 1),
                sound_event,
            },
            sound_category: VarInt(sound_category as i32),
            entity_id,
            volume,
            pitch,
            seed,
        }
    }
}
