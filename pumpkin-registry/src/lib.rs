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
use pumpkin_protocol::{client::config::RegistryEntry, codec::identifier::Identifier};
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
    pub registry_id: Identifier,
    pub registry_entries: Vec<RegistryEntry>,
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
    pub fn name(&self) -> Identifier {
        match self {
            Self::Overworld => Identifier::vanilla("overworld"),
            Self::OverworldCaves => Identifier::vanilla("overworld_caves"),
            Self::TheEnd => Identifier::vanilla("the_end"),
            Self::TheNether => Identifier::vanilla("the_nether"),
        }
    }
}

impl Registry {
    pub fn get_synced() -> Vec<Self> {
        let registry_entries = SYNCED_REGISTRIES
            .biome
            .iter()
            .map(|s| RegistryEntry {
                entry_id: Identifier::vanilla(s.0),
                data: Some(pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap()),
            })
            .collect();
        let biome = Registry {
            registry_id: Identifier::vanilla("worldgen/biome"),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .chat_type
            .iter()
            .map(|s| RegistryEntry {
                entry_id: Identifier::vanilla(s.0),
                data: Some(pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap()),
            })
            .collect();
        let chat_type = Registry {
            registry_id: Identifier::vanilla("chat_type"),
            registry_entries,
        };

        // let registry_entries = SYNCED_REGISTRIES
        //     .trim_pattern
        //     .iter()
        //     .map(|s| RegistryEntry {
        //         entry_id: Identifier::vanilla(s.0),
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
        //         entry_id: Identifier::vanilla(s.0),
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
                    entry_id: Identifier::vanilla(s.0),
                    data: Some(pumpkin_nbt::serializer::to_bytes_unnamed(&variant).unwrap()),
                }
            })
            .collect();
        let wolf_variant = Registry {
            registry_id: Identifier::vanilla("wolf_variant"),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .painting_variant
            .iter()
            .map(|s| RegistryEntry {
                entry_id: Identifier::vanilla(s.0),
                data: Some(pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap()),
            })
            .collect();
        let painting_variant = Registry {
            registry_id: Identifier::vanilla("painting_variant"),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .dimension_type
            .iter()
            .map(|s| RegistryEntry {
                entry_id: Identifier::vanilla(s.0),
                data: Some(pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap()),
            })
            .collect();
        let dimension_type = Registry {
            registry_id: Identifier::vanilla("dimension_type"),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .damage_type
            .iter()
            .map(|s| RegistryEntry {
                entry_id: Identifier::vanilla(s.0),
                data: Some(pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap()),
            })
            .collect();
        let damage_type = Registry {
            registry_id: Identifier::vanilla("damage_type"),
            registry_entries,
        };

        let registry_entries = SYNCED_REGISTRIES
            .banner_pattern
            .iter()
            .map(|s| RegistryEntry {
                entry_id: Identifier::vanilla(s.0),
                data: Some(pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap()),
            })
            .collect();
        let banner_pattern = Registry {
            registry_id: Identifier::vanilla("banner_pattern"),
            registry_entries,
        };

        // TODO
        // let registry_entries = SYNCED_REGISTRIES
        //     .enchantment
        //     .iter()
        //     .map(|s| RegistryEntry {
        //         entry_id: Identifier::vanilla(s.0),
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
                entry_id: Identifier::vanilla(s.0),
                data: Some(pumpkin_nbt::serializer::to_bytes_unnamed(&s.1).unwrap()),
            })
            .collect();
        let jukebox_song = Registry {
            registry_id: Identifier::vanilla("jukebox_song"),
            registry_entries,
        };

        // let registry_entries = SYNCED_REGISTRIES
        //     .instrument
        //     .iter()
        //     .map(|s| RegistryEntry {
        //         entry_id: Identifier::vanilla(s.0),
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
