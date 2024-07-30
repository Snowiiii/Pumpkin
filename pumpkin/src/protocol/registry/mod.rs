use serde::Serialize;

use super::{bytebuf::buffer::ByteBuffer, nbt};

mod biomes;
mod chat_type;
mod damage_type;
mod dimensions;

#[derive(Debug, Clone, Serialize)]
struct LoginInfo {
    #[serde(rename = "minecraft:dimension_type")]
    dimensions: Codec<dimensions::Dimension>,
    #[serde(rename = "minecraft:worldgen/biome")]
    biomes: Codec<biomes::Biome>,
    #[serde(rename = "minecraft:chat_type")]
    chat: Codec<chat_type::ChatType>,
    #[serde(rename = "minecraft:damage_type")]
    damage: Codec<damage_type::DamageType>,
}

#[derive(Debug, Clone, Serialize)]
struct Codec<T> {
    #[serde(rename = "type")]
    ty: String,
    value: Vec<CodecItem<T>>,
}
#[derive(Debug, Clone, Serialize)]
struct CodecItem<T> {
    name: String,
    id: i32,
    element: T,
}

pub fn write_single_dimension<T>(out: &mut ByteBuffer, world_min_y: i32, world_height: u32)
where
    std::io::Cursor<T>: std::io::Write,
{
    let dimension = dimensions::overworld(world_min_y, world_height);
    out.write_bytes(&nbt::to_nbt("", &dimension).unwrap().serialize());
}

pub fn write_codec(out: &mut ByteBuffer, world_min_y: i32, world_height: u32) {
    let dimension = dimensions::overworld(world_min_y, world_height);

    let info = LoginInfo {
        dimensions: Codec {
            ty: "minecraft:dimension_type".into(),
            value: vec![CodecItem {
                name: "minecraft:overworld".into(),
                id: 0,
                element: dimension,
            }],
        },
        biomes: Codec {
            ty: "minecraft:worldgen/biome".into(),
            value: biomes::all(),
        },
        chat: Codec {
            ty: "minecraft:chat_type".into(),
            value: chat_type::all(),
        },
        damage: Codec {
            ty: "minecraft:damage_type".into(),
            value: damage_type::all(),
        },
    };

    // Dimension codec
    out.write_bytes(&nbt::to_nbt("", &info).unwrap().serialize());
    // Current dimension type (key in dimension codec)
    out.write_string("minecraft:overworld");
    // Current world
    out.write_string("minecraft:overworld");
}
