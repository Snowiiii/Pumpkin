use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

const ITEMS_JSON: &str = include_str!("../../../assets/items.json");

pub static ITEMS: LazyLock<HashMap<String, Item>> = LazyLock::new(|| {
    serde_json::from_str(ITEMS_JSON).expect("Could not parse items.json registry.")
});

pub static ITEMS_BY_ID: LazyLock<Vec<&Item>> = LazyLock::new(|| {
    let mut vec = Vec::with_capacity(ITEMS.len());
    for (_, item) in ITEMS.iter() {
        vec.push(item);
    }
    vec.sort_by_key(|it| it.id);
    vec
});

pub fn get_item(name: &str) -> Option<&Item> {
    ITEMS.get(name)
}

pub fn get_item_by_id(id: u16) -> Option<&'static Item> {
    ITEMS_BY_ID.get(id as usize).map(|it| *it)
}

#[derive(Deserialize, Clone, Debug)]
pub struct Item {
    pub id: u16,
    pub block_id: u16,
    pub components: ItemComponents,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ItemComponents {
    #[serde(rename = "minecraft:max_stack_size")]
    pub max_stack_size: u8,
}
