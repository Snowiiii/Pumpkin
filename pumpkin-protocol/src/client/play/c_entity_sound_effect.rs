use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::{SoundCategory, VarInt};

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::EntitySoundEffect as i32)]
pub struct CEntitySoundEffect {
    sound_id: VarInt,
    // TODO: add sound from name
    // sound_name: Option<&'a str>,
    // has_fixed_range: Option<bool>,
    // range: Option<f32>,
    sound_category: VarInt,
    entity_id: VarInt,
    volume: f32,
    pitch: f32,
    seed: f64,
}

impl CEntitySoundEffect {
    pub fn new(
        sound_id: VarInt,
        sound_category: SoundCategory,
        entity_id: VarInt,
        volume: f32,
        pitch: f32,
        seed: f64,
    ) -> Self {
        Self {
            sound_id: VarInt(sound_id.0 + 1),
            sound_category: VarInt(sound_category as i32),
            entity_id,
            volume,
            pitch,
            seed,
        }
    }
}
