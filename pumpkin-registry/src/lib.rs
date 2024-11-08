use std::{collections::HashMap, sync::LazyLock};

use banner_pattern::BannerPattern;
use biome::Biome;
use chat_type::ChatType;
use damage_type::DamageType;
use dimension::Dimension;
use enchantment::Enchantment;
use fastnbt::SerOpts;
use instrument::Instrument;
use jukebox_song::JukeboxSong;
use paint::Painting;
use pumpkin_protocol::client::config::RegistryEntry;
use serde::{Deserialize, Serialize};
use trim_material::TrimMaterial;
use trim_pattern::TrimPattern;
use wolf::WolfVariant;

mod banner_pattern;
mod biome;
mod chat_type;
mod damage_type;
mod dimension;
mod enchantment;
mod instrument;
mod jukebox_song;
mod paint;
mod trim_material;
mod trim_pattern;
mod wolf;

pub static SYNCED_REGISTRIES: LazyLock<SyncedRegistry> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/synced_registries.json"))
        .expect("Could not parse synced_registries.json registry.")
});

pub struct Registry {
    pub registry_id: String,
    pub registry_entries: Vec<RegistryEntry<'static>>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncedRegistry {
    #[serde(rename = "minecraft:worldgen/biome")]
    biome: HashMap<String, Biome>,
    #[serde(rename = "minecraft:chat_type")]
    chat_type: HashMap<String, ChatType>,
    #[serde(rename = "minecraft:trim_pattern")]
    trim_pattern: HashMap<String, TrimPattern>,
    #[serde(rename = "minecraft:trim_material")]
    trim_material: HashMap<String, TrimMaterial>,
    #[serde(rename = "minecraft:wolf_variant")]
    wolf_variant: HashMap<String, WolfVariant>,
    #[serde(rename = "minecraft:painting_variant")]
    painting_variant: HashMap<String, Painting>,
    #[serde(rename = "minecraft:dimension_type")]
    dimension_type: HashMap<String, Dimension>,
    #[serde(rename = "minecraft:damage_type")]
    damage_type: HashMap<String, DamageType>,
    #[serde(rename = "minecraft:banner_pattern")]
    banner_pattern: HashMap<String, BannerPattern>,
    #[serde(rename = "minecraft:enchantment")]
    enchantment: HashMap<String, Enchantment>,
    #[serde(rename = "minecraft:jukebox_song")]
    jukebox_song: HashMap<String, JukeboxSong>,
    #[serde(rename = "minecraft:instrument")]
    instrument: HashMap<String, Instrument>,
}

impl Registry {
    pub fn get_synced() -> Vec<Self> {
        let registry_entries = SYNCED_REGISTRIES
            .biome
            .iter()
            .map(|s| RegistryEntry {
                entry_id: s.0,
                data: fastnbt::to_bytes_with_opts(&s.1, SerOpts::network_nbt()).unwrap(),
            })
            .collect();
        let biome = Registry {
            registry_id: "minecraft:worldgen/biome".to_string(),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .chat_type
            .iter()
            .map(|s| RegistryEntry {
                entry_id: s.0,
                data: fastnbt::to_bytes_with_opts(&s.1, SerOpts::network_nbt()).unwrap(),
            })
            .collect();
        let chat_type = Registry {
            registry_id: "minecraft:chat_type".to_string(),
            registry_entries,
        };

        // let registry_entries = SYNCED_REGISTRIES
        //     .trim_pattern
        //     .iter()
        //     .map(|s| RegistryEntry {
        //         entry_id: s.0,
        //         data: fastnbt::to_bytes_with_opts(&s.1, SerOpts::network_nbt()).unwrap(),
        //     })
        //     .collect();
        // let trim_pattern = Registry {
        //     registry_id: "minecraft:trim_pattern".to_string(),
        //     registry_entries,
        // };

        // let registry_entries = SYNCED_REGISTRIES
        //     .trim_material
        //     .iter()
        //     .map(|s| RegistryEntry {
        //         entry_id: s.0,
        //         data: fastnbt::to_bytes_with_opts(&s.1, SerOpts::network_nbt()).unwrap(),
        //     })
        //     .collect();
        // let trim_material = Registry {
        //     registry_id: "minecraft:trim_material".to_string(),
        //     registry_entries,
        // };

        let registry_entries = SYNCED_REGISTRIES
            .wolf_variant
            .iter()
            .map(|s| {
                // I present to you, A ugly hack which is done because Mojang developers decited to put is_<biome> instead of just <biome> on 3 wolf varients while all others have just the biome, this causes the client to not find the biome and disconnect
                let varient = s.1.clone();
                RegistryEntry {
                    entry_id: s.0,
                    data: fastnbt::to_bytes_with_opts(&varient, SerOpts::network_nbt()).unwrap(),
                }
            })
            .collect();
        let wolf_variant = Registry {
            registry_id: "minecraft:wolf_variant".to_string(),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .painting_variant
            .iter()
            .map(|s| RegistryEntry {
                entry_id: s.0,
                data: fastnbt::to_bytes_with_opts(&s.1, SerOpts::network_nbt()).unwrap(),
            })
            .collect();
        let painting_variant = Registry {
            registry_id: "minecraft:painting_variant".to_string(),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .dimension_type
            .iter()
            .map(|s| RegistryEntry {
                entry_id: s.0,
                data: fastnbt::to_bytes_with_opts(&s.1, SerOpts::network_nbt()).unwrap(),
            })
            .collect();
        let dimension_type = Registry {
            registry_id: "minecraft:dimension_type".to_string(),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .damage_type
            .iter()
            .map(|s| RegistryEntry {
                entry_id: s.0,
                data: fastnbt::to_bytes_with_opts(&s.1, SerOpts::network_nbt()).unwrap(),
            })
            .collect();
        let damage_type = Registry {
            registry_id: "minecraft:damage_type".to_string(),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .banner_pattern
            .iter()
            .map(|s| RegistryEntry {
                entry_id: s.0,
                data: fastnbt::to_bytes_with_opts(&s.1, SerOpts::network_nbt()).unwrap(),
            })
            .collect();
        let banner_pattern = Registry {
            registry_id: "minecraft:banner_pattern".to_string(),
            registry_entries,
        };

        // TODO
        // let registry_entries = SYNCED_REGISTRIES
        //     .enchantment
        //     .iter()
        //     .map(|s| RegistryEntry {
        //         entry_id: s.0,
        //         data: fastnbt::to_bytes_with_opts(&s.1, SerOpts::network_nbt()).unwrap(),
        //     })
        //     .collect();
        // let enchantment = Registry {
        //     registry_id: "minecraft:enchantment".to_string(),
        //     registry_entries,
        // };

        // let registry_entries = SYNCED_REGISTRIES
        //     .jukebox_song
        //     .iter()
        //     .map(|s| RegistryEntry {
        //         entry_id: s.0,
        //         data: fastnbt::to_bytes_with_opts(&s.1, SerOpts::network_nbt()).unwrap(),
        //     })
        //     .collect();
        // let jukebox_song = Registry {
        //     registry_id: "minecraft:jukebox_song".to_string(),
        //     registry_entries,
        // };

        // let registry_entries = SYNCED_REGISTRIES
        //     .instrument
        //     .iter()
        //     .map(|s| RegistryEntry {
        //         entry_id: s.0,
        //         data: fastnbt::to_bytes_with_opts(&s.1, SerOpts::network_nbt()).unwrap(),
        //     })
        //     .collect();
        // let instrument = Registry {
        //     registry_id: "minecraft:instrument".to_string(),
        //     registry_entries,
        // };

        vec![
            biome,
            chat_type,
            // trim_pattern,
            // trim_material,
            wolf_variant,
            painting_variant,
            dimension_type,
            damage_type,
            banner_pattern,
            // enchantment,
            // jukebox_song,
            // instrument,
        ]
    }
}
