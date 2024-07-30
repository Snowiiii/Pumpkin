use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Dimension {
    ambient_light: f32,
    bed_works: bool,
    coordinate_scale: f32,
    effects: String,
    has_ceiling: bool,
    has_raids: bool,
    has_skylight: bool,
    height: i32, // 1.17+
    infiniburn: String,
    logical_height: i32,
    min_y: i32, // 1.17+
    natural: bool,
    piglin_safe: bool,
    fixed_time: i64,
    respawn_anchor_works: bool,
    ultrawarm: bool,

    // 1.19+
    monster_spawn_light_level: i32,
    monster_spawn_block_light_limit: i32,
}

pub fn overworld(world_min_y: i32, world_height: u32) -> Dimension {
    Dimension {
        piglin_safe: false,
        natural: true,
        ambient_light: 0.0,
        fixed_time: 6000,
        infiniburn: "#minecraft:infiniburn_overworld".into(),
        respawn_anchor_works: false,
        has_skylight: true,
        bed_works: true,
        effects: "minecraft:overworld".into(),
        has_raids: false,
        logical_height: 128,
        coordinate_scale: 1.0,
        ultrawarm: false,
        has_ceiling: false,
        min_y: world_min_y,
        height: (world_height as i32 + 15) / 16 * 16,

        monster_spawn_light_level: 7,
        monster_spawn_block_light_limit: 7,
    }
}
