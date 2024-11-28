use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    ambient_light: f32,
    bed_works: bool,
    coordinate_scale: f64,
    effects: DimensionEffects,
    #[serde(skip_serializing_if = "Option::is_none")]
    fixed_time: Option<i64>,
    has_ceiling: bool,
    has_raids: bool,
    has_skylight: bool,
    height: i32,
    infiniburn: String,
    logical_height: i32,
    min_y: i32,
    monster_spawn_block_light_limit: i32,
    monster_spawn_light_level: MonsterSpawnLightLevel,
    natural: bool,
    piglin_safe: bool,
    respawn_anchor_works: bool,
    ultrawarm: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default, Debug)]
pub enum DimensionEffects {
    #[serde(rename = "minecraft:overworld")]
    #[default]
    Overworld,
    #[serde(rename = "minecraft:the_nether")]
    TheNether,
    #[serde(rename = "minecraft:the_end")]
    TheEnd,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MonsterSpawnLightLevel {
    Int(i32),
    Tagged(MonsterSpawnLightLevelTagged),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MonsterSpawnLightLevelTagged {
    min_inclusive: i32,
    max_inclusive: i32,
    #[serde(rename = "type")]
    typee: String,
}

impl From<i32> for MonsterSpawnLightLevel {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}
