use crate::protocol::VarInt;

use super::CodecItem;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Biome {
    has_precipitation: bool,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature_modifier: Option<String>,
    downfall: f32,
    effects: BiomeEffects,
}
#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
struct Particle {
    options: ParticleOptions,
    probability: f32,
}

#[derive(Debug, Clone, Serialize)]
struct ParticleOptions {
    typee: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<VarInt>,
}

#[derive(Debug, Clone, Serialize)]
struct MoodSound {
    block_search_extent: i32,
    offset: f64,
    sound: String,
    tick_delay: i32,
}

#[derive(Debug, Clone, Serialize)]
struct AdditionsSound {
    sound: String,
    tick_chance: f64,
}

#[derive(Debug, Clone, Serialize)]
struct Music {
    sound: String,
    min_delay: i32,
    max_delay: i32,
    replace_current_music: bool,
}

// 1.20.6 default https://gist.github.com/WinX64/ab8c7a8df797c273b32d3a3b66522906
pub(super) fn all() -> Vec<CodecItem<Biome>> {
    let biome = Biome {
        has_precipitation: false,
        temperature: 1.0,
        temperature_modifier: None,
        downfall: 0.0,
        effects: BiomeEffects {
            fog_color: 0x7FA1FF,
            water_color: 0x7FA1FF,
            water_fog_color: 0x7FA1FF,
            sky_color: 0x7FA1FF,
            foliage_color: Some(0x7FA1FF),
            grass_color: Some(0x7FA1FF),
            grass_color_modifier: None,
            particle: None,
            ambient_sound: None,
            mood_sound: Some(MoodSound {
                block_search_extent: 8,
                offset: 2.0,
                sound: "minecraft:ambient.cave".into(),
                tick_delay: 6000,
            }),
            additions_sound: None,
            music: None,
        },
    };

    vec![CodecItem {
        name: "minecraft:plains".into(),
        id: 0,
        element: biome,
    }]
}
