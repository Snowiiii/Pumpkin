use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Dimension {
    #[serde(skip_serializing_if = "Option::is_none")]
    fixed_time: Option<f64>,
    has_skylight: bool,
    has_ceiling: bool,
    ultrawarm: bool,
    natural: bool,
    coordinate_scale: f64,
    bed_works: bool,
    respawn_anchor_works: bool,
    min_y: i32,
    height: i32,
    logical_height: i32,
    infiniburn: String,
    effects: String,
    ambient_light: f32,
    piglin_safe: bool,
    has_raids: bool,
    monster_spawn_light_level: i32,
    monster_spawn_block_light_limit: i32,
}

pub fn overworld(world_min_y: i32, world_height: u32) -> Dimension {
    Dimension {
        piglin_safe: false,
        natural: true,
        ambient_light: 0.0,
        fixed_time: Some(6000.0),
        infiniburn: "#minecraft:infiniburn_overworld".into(),
        respawn_anchor_works: false,
        has_skylight: true,
        bed_works: true,
        effects: "minecraft:overworld".into(),
        has_raids: false,
        logical_height: 384,
        coordinate_scale: 1.0,
        ultrawarm: false,
        has_ceiling: false,
        min_y: world_min_y,
        height: (world_height as i32 + 15) / 16 * 16,

        monster_spawn_light_level: 7,
        monster_spawn_block_light_limit: 7,
    }
}
