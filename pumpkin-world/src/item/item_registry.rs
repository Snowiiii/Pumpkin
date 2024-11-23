use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

const ITEMS_JSON: &str = include_str!("../../../assets/items.json");

pub static ITEMS: LazyLock<HashMap<String, Item>> = LazyLock::new(|| {
    serde_json::from_str(ITEMS_JSON).expect("Could not parse items.json registry.")
});

pub fn get_item(name: &str) -> Option<&Item> {
    ITEMS.get(name)
}

pub fn get_item_by_id<'a>(id: u16) -> Option<&'a Item> {
    let item = ITEMS.iter().find(|item| item.1.id == id);
    if let Some(item) = item {
        return Some(item.1)
    }
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
    #[serde(rename = "minecraft:jukebox_playable")]
    pub jukebox_playable: Option<JukeboxPlayable>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct JukeboxPlayable {
    pub song: String,
}
