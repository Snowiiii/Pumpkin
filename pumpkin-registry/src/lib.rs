use crate::damage_type::DamageType;
use biomes::Biome;
use chat_type::ChatType;
use dimensions::Dimension;
use fastnbt::SerOpts;
use paint::Painting;
use pumpkin_protocol::client::config::RegistryEntry;
use serde::Serialize;
use wolf::WolfVariant;

mod biomes;
mod chat_type;
mod damage_type;
mod dimensions;
mod paint;
mod wolf;

pub struct Registry {
    pub registry_id: &'static str,
    pub registry_entries: Vec<RegistryEntry<'static>>,
}

fn get_entry<T: Serialize + Default>(entry_id: &'static str) -> RegistryEntry<'static> {
    RegistryEntry {
        entry_id,
        data: fastnbt::to_bytes_with_opts(&T::default(), SerOpts::network_nbt()).unwrap(),
    }
}
trait RegistryType: Serialize + Default {
    const REGISTRY_ID: &'static str;
    const ENTRY_IDS: &'static [&'static str];

    fn registry() -> Registry {
        let entries = Self::ENTRY_IDS
            .iter()
            .map(|entry| get_entry::<Self>(entry))
            .collect();
        Registry {
            registry_id: Self::REGISTRY_ID,
            registry_entries: entries,
        }
    }
}

impl Registry {
    /// We should parse this from a JSON in the future
    pub fn get_static() -> Vec<Self> {
        vec![
            Dimension::registry(),
            DamageType::registry(),
            Biome::registry(),
            WolfVariant::registry(),
            Painting::registry(),
            ChatType::registry(),
        ]
    }
}
