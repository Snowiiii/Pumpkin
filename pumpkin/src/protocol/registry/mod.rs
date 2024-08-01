use dimensions::Dimension;
use fastnbt::SerOpts;
use serde::Serialize;
use wolf::WolfVariant;

use super::bytebuf::ByteBuffer;

mod biomes;
mod chat_type;
mod damage_type;
mod dimensions;
mod wolf;

#[derive(Debug, Clone, Serialize)]
struct LoginInfo {
    #[serde(rename = "minecraft:dimension_type")]
    dimensions: Codec<dimensions::Dimension>,
    // #[serde(rename = "minecraft:wolf_variant")]
    // wolf: Codec<wolf::WolfVariant>,
    /*   #[serde(rename = "minecraft:worldgen/biome")]
    biomes: Codec<biomes::Biome>,
    #[serde(rename = "minecraft:chat_type")]
    chat: Codec<chat_type::ChatType>,
    #[serde(rename = "minecraft:damage_type")]
    damage: Codec<damage_type::DamageType>, */
}

#[derive(Debug, Clone, Serialize)]
pub struct DimensionCodec {
    #[serde(rename = "minecraft:dimension_type")]
    dimensions: Codec<dimensions::Dimension>,
}

impl DimensionCodec {
    pub fn parse() -> Vec<u8> {
        let dimension = Dimension::default();
        let codec = DimensionCodec {
            dimensions: Codec {
                ty: "minecraft:dimension_type".into(),
                value: vec![RegistryValue {
                    name: "minecraft:overworld".into(),
                    id: 0,
                    element: dimension,
                }],
            },
        };
        fastnbt::to_bytes_with_opts(&codec, SerOpts::network_nbt()).unwrap()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WolfCodec {
    #[serde(rename = "minecraft:wolf_variant")]
    wolf: Codec<wolf::WolfVariant>,
}

impl WolfCodec {
    pub fn parse() -> Vec<u8> {
        let wolf = WolfVariant::default();
        let codec = WolfCodec {
            wolf: Codec {
                ty: "minecraft:wolf_variant".into(),
                value: vec![RegistryValue {
                    name: "minecraft:wolf_variant".into(),
                    id: 0,
                    element: wolf,
                }],
            },
        };
        fastnbt::to_bytes_with_opts(&codec, SerOpts::network_nbt()).unwrap()
    }
}

#[derive(Debug, Clone, Serialize)]
struct Codec<T> {
    #[serde(rename = "type")]
    ty: String,
    value: Vec<RegistryValue<T>>,
}
#[derive(Debug, Clone, Serialize)]
struct RegistryValue<T> {
    name: String,
    id: i32,
    element: T,
}

pub fn write_codec(out: &mut ByteBuffer, world_min_y: i32, world_height: u32) -> Vec<u8> {
    let dimension = Dimension::default();
    let wolf = WolfVariant::default();

    let info = LoginInfo {
        dimensions: Codec {
            ty: "minecraft:dimension_type".into(),
            value: vec![RegistryValue {
                name: "minecraft:overworld".into(),
                id: 0,
                element: dimension,
            }],
        },
        /* wolf: Codec {
         ty: "minecraft:wolf_variant".into(),
         value: vec![RegistryValue {
             name: "minecraft:wolf_variant".into(),
             id: 0,
             element: wolf,
         }],
         }, */ /* biomes: Codec {
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
            }, */
    };

    fastnbt::to_bytes_with_opts(&info, SerOpts::network_nbt()).unwrap()
}
