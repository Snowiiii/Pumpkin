use pumpkin_protocol::VarInt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Biome {
    has_precipitation: bool,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature_modifier: Option<String>,
    downfall: f32,
    effects: BiomeEffects,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BiomeEffects {
    fog_color: i32,
    water_color: i32,
    water_fog_color: i32,
    sky_color: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    foliage_color: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    grass_color: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    grass_color_modifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    particle: Option<Particle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ambient_sound: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mood_sound: Option<MoodSound>,
    #[serde(skip_serializing_if = "Option::is_none")]
    additions_sound: Option<AdditionsSound>,
    #[serde(skip_serializing_if = "Option::is_none")]
    music: Option<Music>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Particle {
    options: ParticleOptions,
    probability: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParticleOptions {
    #[serde(rename = "type")]
    typee: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<VarInt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MoodSound {
    block_search_extent: i32,
    offset: f64,
    sound: String,
    tick_delay: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdditionsSound {
    sound: String,
    tick_chance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Music {
    sound: String,
    min_delay: i32,
    max_delay: i32,
    replace_current_music: bool,
}
