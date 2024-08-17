use std::collections::HashMap;

use lazy_static::lazy_static;

pub const ITEM_REGISTRY: &str = "minecraft:item";

const REGISTRY_JSON: &str = include_str!("../assets/registries.json");

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RegistryElement {
    default: Option<String>,
    entries: HashMap<String, u32>,
}

lazy_static! {
    static ref REGISTRY: HashMap<String, RegistryElement> =
        serde_json::from_str(REGISTRY_JSON).expect("Could not parse items.json registry.");
}

pub fn get_protocol_id(category: &str, entry: &str) -> u32 {
    *REGISTRY
        .get(category)
        .expect("Invalid Category in registry")
        .entries
        .get(entry)
        .expect("No Entry found")
}

pub fn get_default<'a>(category: &str) -> Option<&'a str> {
    REGISTRY
        .get(category)
        .expect("Invalid Category in registry")
        .default
        .as_deref()
}
