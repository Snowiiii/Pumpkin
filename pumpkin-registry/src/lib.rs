use std::sync::LazyLock;

use banner_pattern::BannerPattern;
use biome::Biome;
use chat_type::ChatType;
use damage_type::DamageType;
use dimension::Dimension;
use enchantment::Enchantment;
use indexmap::IndexMap;
use instrument::Instrument;
use jukebox_song::JukeboxSong;
use paint::Painting;
use pumpkin_protocol::client::config::RegistryEntry;
pub use recipe::{
    flatten_3x3, IngredientSlot, IngredientType, Recipe, RecipeResult, RecipeType, RECIPES,
};
use serde::{Deserialize, Serialize};
pub use tags::{get_tag_values, TagCategory, TagType};
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
mod recipe;
mod tags;
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
    #[serde(rename = "worldgen/biome")]
    biome: IndexMap<String, Biome>,
    chat_type: IndexMap<String, ChatType>,
    trim_pattern: IndexMap<String, TrimPattern>,
    trim_material: IndexMap<String, TrimMaterial>,
    wolf_variant: IndexMap<String, WolfVariant>,
    painting_variant: IndexMap<String, Painting>,
    dimension_type: IndexMap<String, Dimension>,
    damage_type: IndexMap<String, DamageType>,
    banner_pattern: IndexMap<String, BannerPattern>,
    enchantment: IndexMap<String, Enchantment>,
    pub jukebox_song: IndexMap<String, JukeboxSong>,
    instrument: IndexMap<String, Instrument>,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum DimensionType {
    Overworld,
    OverworldCaves,
    TheEnd,
    TheNether,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataPool<T> {
    data: T,
    weight: i32,
}

impl DimensionType {
    pub fn name(&self) -> &str {
        match self {
            Self::Overworld => "minecraft:overworld",
            Self::OverworldCaves => "minecraft:overworld_caves",
            Self::TheEnd => "minecraft:the_end",
            Self::TheNether => "minecraft:the_nether",
        }
    }
}

impl Registry {
    pub fn get_synced() -> Vec<Self> {
        let registry_entries = SYNCED_REGISTRIES
            .biome
            .iter()
            .map(|s| RegistryEntry {
                entry_id: s.0,
                data: pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap(),
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
                data: pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap(),
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
        //         data: pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap(),
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
        //         data: pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap(),
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
                let variant = s.1.clone();
                RegistryEntry {
                    entry_id: s.0,
                    data: pumpkin_nbt::serializer::to_bytes_unnamed(&variant).unwrap(),
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
                data: pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap(),
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
                data: pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap(),
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
                data: pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap(),
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
                data: pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap(),
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
        //         data: pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap(),
        //     })
        //     .collect();
        // let enchantment = Registry {
        //     registry_id: "minecraft:enchantment".to_string(),
        //     registry_entries,
        // };

        let registry_entries = SYNCED_REGISTRIES
            .jukebox_song
            .iter()
            .map(|s| RegistryEntry {
                entry_id: s.0,
                data: pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap(),
            })
            .collect();
        let jukebox_song = Registry {
            registry_id: "minecraft:jukebox_song".to_string(),
            registry_entries,
        };

        // let registry_entries = SYNCED_REGISTRIES
        //     .instrument
        //     .iter()
        //     .map(|s| RegistryEntry {
        //         entry_id: s.0,
        //         data: pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap(),
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
            jukebox_song,
            // instrument,
        ]
    }
}
