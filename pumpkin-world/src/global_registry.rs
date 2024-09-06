use std::{collections::HashMap, sync::LazyLock};

pub const ITEM_REGISTRY: &str = "minecraft:item";

const REGISTRY_JSON: &str = include_str!("../assets/registries.json");

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RegistryElement {
    default: Option<String>,
    pub entries: HashMap<String, HashMap<String, u32>>,
}

pub static REGISTRY: LazyLock<HashMap<String, RegistryElement>> = LazyLock::new(|| {
    serde_json::from_str(REGISTRY_JSON).expect("Could not parse registry.json registry.")
});

pub fn get_protocol_id(category: &str, entry: &str) -> u32 {
    *REGISTRY
        .get(category)
        .expect("Invalid Category in registry")
        .entries
        .get(entry)
        .map(|p| p.get("protocol_id").unwrap())
        .expect("No Entry found")
}

pub fn get_default<'a>(category: &str) -> Option<&'a str> {
    REGISTRY
        .get(category)
        .expect("Invalid Category in registry")
        .default
        .as_deref()
}

pub fn find_minecraft_id(category: &str, protocol_id: u32) -> Option<&str> {
    REGISTRY
        .get(category)?
        .entries
        .iter()
        .find(|(_, other_protocol_id)| {
            *other_protocol_id.get("protocol_id").unwrap() == protocol_id
        })
        .map(|(id, _)| id.as_str())
}
