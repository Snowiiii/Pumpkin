use std::{collections::HashMap, sync::LazyLock};

use pumpkin_core::registries::blocks::Block;
use serde::Deserialize;

use crate::block::block_registry;

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
    ITEMS_BY_ID.get(id as usize).copied()
}

#[derive(Deserialize, Clone, Debug)]
pub struct Item {
    pub id: u16,
    pub block_id: Option<u16>,
    pub components: ItemComponents,
}

impl Item {
    pub fn get_block(&self) -> Option<&Block> {
        block_registry::get_block_by_id(self.block_id?)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ItemComponents {
    #[serde(rename = "minecraft:max_stack_size")]
    pub max_stack_size: u8,
}
