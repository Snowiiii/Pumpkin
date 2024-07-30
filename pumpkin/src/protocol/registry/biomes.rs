use super::CodecItem;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Biome {
    category: String,
    depth: f32,
    downfall: f32,
    effects: BiomeEffects,
    precipitation: String,
    scale: f32,
    temperature: f32,
    has_precipitation: bool,
}
#[derive(Debug, Clone, Serialize)]
struct BiomeEffects {
    sky_color: i32,
    fog_color: i32,
    water_fog_color: i32,
    water_color: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    foliage_color: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    grass_color: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mood_sound: Option<MoodSound>, // 1.18.2+
}
#[derive(Debug, Clone, Serialize)]
struct MoodSound {
    block_search_extent: i32,
    offset: f64,
    sound: String,
    tick_delay: i32,
}

pub(super) fn all() -> Vec<CodecItem<Biome>> {
    let biome = Biome {
        precipitation: "rain".into(),
        depth: 1.0,
        temperature: 1.0,
        scale: 1.0,
        downfall: 1.0,
        category: "none".into(),
        has_precipitation: true,
        effects: BiomeEffects {
            sky_color: 0x78a7ff,
            fog_color: 0xc0d8ff,
            water_fog_color: 0x050533,
            water_color: 0x3f76e4,
            foliage_color: None,
            grass_color: None,
            mood_sound: Some(MoodSound {
                block_search_extent: 8,
                offset: 2.0,
                sound: "minecraft:ambient.cave".into(),
                tick_delay: 6000,
            }),
            // sky_color:       0xff00ff,
            // water_color:     0xff00ff,
            // fog_color:       0xff00ff,
            // water_fog_color: 0xff00ff,
            // grass_color:     0xff00ff,
            // foliage_color:   0x00ffe5,
            // grass_color:     0xff5900,
        },
    };

    vec![CodecItem {
        name: "minecraft:plains".into(),
        id: 0,
        element: biome,
    }]
}
