use std::{collections::HashMap, sync::LazyLock};

use super::Rarity;
use crate::global_registry::{self, ITEM_REGISTRY};

const ITEMS_JSON: &str = include_str!("../../assets/items.json");

pub static ITEMS: LazyLock<HashMap<String, ItemElement>> = LazyLock::new(|| {
    serde_json::from_str(ITEMS_JSON).expect("Could not parse items.json registry.")
});

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ItemComponents {
    // TODO: attribute_modifiers
    // TODO: enchantments: HashMap<>
    #[serde(rename = "minecraft:lore")]
    lore: Vec<String>,
    #[serde(rename = "minecraft:max_stack_size")]
    max_stack_size: u32,
    #[serde(rename = "minecraft:rarity")]
    rarity: Rarity,
    #[serde(rename = "minecraft:repair_cost")]
    repair_cost: u32,
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ItemElement {
    components: ItemComponents,
}

#[allow(dead_code)]
pub fn get_item_element(item_id: &str) -> &ItemComponents {
    &ITEMS.get(item_id).expect("Item not found").components
}

#[allow(dead_code)]
pub fn get_item_protocol_id(item_id: &str) -> u32 {
    global_registry::get_protocol_id(ITEM_REGISTRY, item_id)
}
