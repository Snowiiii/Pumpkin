use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

const ITEMS_JSON: &str = include_str!("../../../assets/items.json");

pub static ITEMS: LazyLock<HashMap<String, Item>> = LazyLock::new(|| {
    serde_json::from_str(ITEMS_JSON).expect("Could not parse items.json registry.")
});

pub static ITEMS_REGISTRY_ID_BY_ID: LazyLock<HashMap<u16, String>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for item in ITEMS.clone() {
        map.insert(item.1.id, item.0.clone());
    }
    map
});

pub fn get_item(name: &str) -> Option<&Item> {
    ITEMS.get(&name.replace("minecraft:", ""))
}

pub fn get_item_by_id<'a>(item_id: u16) -> Option<&'a Item> {
    ITEMS.values().find(|&item| item.id == item_id)
}

pub fn get_spawn_egg(item_id: u16) -> Option<String> {
    if let Some(item_name) = ITEMS_REGISTRY_ID_BY_ID.get(&item_id) {
        if item_name.ends_with("_spawn_egg") {
            if let Some(res) = item_name.strip_suffix("_spawn_egg") {
                return Some(res.to_owned());
            }
        }
    };
    None
}

#[derive(Deserialize, Clone, Debug)]
pub struct Item {
    pub id: u16,
    pub components: ItemComponents,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ItemComponents {
    #[serde(rename = "minecraft:max_stack_size")]
    pub max_stack_size: u8,
}
