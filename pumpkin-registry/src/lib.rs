use biomes::Biome;
use damage_type::DamageType;
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
    pub registry_entries: Vec<RegistryEntry>,
}

impl Registry {
    pub fn get_static() -> Vec<Self> {
        let dimensions = Registry {
            registry_id: "minecraft:dimension_type".to_string(),
            registry_entries: vec![RegistryEntry {
                entry_id: "minecraft:overworld".to_string(),
                data: fastnbt::to_bytes_with_opts(&Dimension::default(), SerOpts::network_nbt())
                    .unwrap(),
            }],
        };
        let biomes = Registry {
            registry_id: "minecraft:worldgen/biome".to_string(),
            registry_entries: vec![
                RegistryEntry {
                    entry_id: "minecraft:snowy_taiga".to_string(),
                    data: fastnbt::to_bytes_with_opts(&Biome::default(), SerOpts::network_nbt())
                        .unwrap(),
                },
                RegistryEntry {
                    entry_id: "minecraft:plains".to_string(),
                    data: fastnbt::to_bytes_with_opts(&Biome::default(), SerOpts::network_nbt())
                        .unwrap(),
                },
            ],
        };
        let wolf_variants = Registry {
            registry_id: "minecraft:wolf_variant".to_string(),
            registry_entries: vec![RegistryEntry {
                entry_id: "minecraft:wolf_variant".to_string(),
                data: fastnbt::to_bytes_with_opts(&WolfVariant::default(), SerOpts::network_nbt())
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
                entry_id: "minecraft:painting_variant".to_string(),
                data: fastnbt::to_bytes_with_opts(&Painting::default(), SerOpts::network_nbt())
                    .unwrap(),
            }],
        };
        vec![dimensions, damage_types, biomes, wolf_variants, paintings]
    }
}
