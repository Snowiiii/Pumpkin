use biomes::Biome;
use chat_type::ChatType;
use dimensions::Dimension;
use fastnbt::SerOpts;
use paint::Painting;
use pumpkin_protocol::client::config::RegistryEntry;
use wolf::WolfVariant;

mod biomes;
mod chat_type;
mod damage_type;
mod dimensions;
mod paint;
mod wolf;

pub struct Registry {
    pub registry_id: String,
    pub registry_entries: Vec<RegistryEntry<'static>>,
}

impl Registry {
    /// We should parse this from a JSON in the future
    pub fn get_static() -> Vec<Self> {
        let dimensions = Registry {
            registry_id: "minecraft:dimension_type".to_string(),
            registry_entries: vec![RegistryEntry {
                entry_id: "minecraft:overworld",
                data: fastnbt::to_bytes_with_opts(&Dimension::default(), SerOpts::network_nbt())
                    .unwrap(),
            }],
        };
        let biomes = Registry {
            registry_id: "minecraft:worldgen/biome".to_string(),
            registry_entries: vec![
                RegistryEntry {
                    entry_id: "minecraft:snowy_taiga",
                    data: fastnbt::to_bytes_with_opts(&Biome::default(), SerOpts::network_nbt())
                        .unwrap(),
                },
                RegistryEntry {
                    entry_id: "minecraft:plains",
                    data: fastnbt::to_bytes_with_opts(&Biome::default(), SerOpts::network_nbt())
                        .unwrap(),
                },
            ],
        };
        let wolf_variants = Registry {
            registry_id: "minecraft:wolf_variant".to_string(),
            registry_entries: vec![RegistryEntry {
                entry_id: "minecraft:wolf_variant",
                data: fastnbt::to_bytes_with_opts(&WolfVariant::default(), SerOpts::network_nbt())
                    .unwrap(),
            }],
        };

        let chat_types = Registry {
            registry_id: "minecraft:chat_type".to_string(),
            registry_entries: vec![RegistryEntry {
                entry_id: "minecraft:chat",
                data: fastnbt::to_bytes_with_opts(&ChatType::default(), SerOpts::network_nbt())
                    .unwrap(),
            }],
        };

        let damage_types = Registry {
            registry_id: "minecraft:damage_type".to_string(),
            registry_entries: damage_type::entires(),
        };
        let paintings = Registry {
            registry_id: "minecraft:painting_variant".to_string(),
            registry_entries: vec![RegistryEntry {
                entry_id: "minecraft:painting_variant",
                data: fastnbt::to_bytes_with_opts(&Painting::default(), SerOpts::network_nbt())
                    .unwrap(),
            }],
        };
        vec![
            dimensions,
            damage_types,
            biomes,
            wolf_variants,
            paintings,
            chat_types,
        ]
    }
}
