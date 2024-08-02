use pumpkin_protocol::VarInt;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BiomeCodec {
    name: String,
    id: i32,
    element: Biome,
}

impl Default for BiomeCodec {
    fn default() -> Self {
        Self {
            name: "minecraft:plains".to_string(),
            id: 0,
            element: Biome::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Biome {
    has_precipitation: i8,
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
    replace_current_music: i8,
}

// 1.20.6 default https://gist.github.com/WinX64/ab8c7a8df797c273b32d3a3b66522906
impl Default for Biome {
    fn default() -> Self {
        Self {
            has_precipitation: 0,
            temperature: 1.0,
            temperature_modifier: None,
            downfall: 0.0,
            effects: BiomeEffects {
                fog_color: 12638463,
                water_color: 4159204,
                water_fog_color: 329011,
                sky_color: 7907327,
                foliage_color: None,
                grass_color: None,
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
        }
    }
}
