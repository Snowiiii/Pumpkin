use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;

const ITEMS_JSON: &str = include_str!("../../../assets/items.json");

pub static ITEMS: LazyLock<HashMap<String, Item>> = LazyLock::new(|| {
    serde_json::from_str(ITEMS_JSON).expect("Could not parse items.json registry.")
});

pub static ITEMS_REGISTRY_NAME_BY_ID: LazyLock<HashMap<u16, String>> = LazyLock::new(|| {
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
    if let Some(item_name) = ITEMS_REGISTRY_NAME_BY_ID.get(&item_id) {
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
    #[serde(rename = "minecraft:jukebox_playable")]
    pub jukebox_playable: Option<JukeboxPlayable>,
    #[serde(rename = "minecraft:damage")]
    pub damage: Option<u16>,
    #[serde(rename = "minecraft:max_damage")]
    pub max_damage: Option<u16>,
    #[serde(rename = "minecraft:attribute_modifiers")]
    pub attribute_modifiers: Option<AttributeModifiers>,
    #[serde(rename = "minecraft:food")]
    pub food: Option<Food>,
    #[serde(rename = "minecraft:equippable")]
    pub equippable: Option<Equippable>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct JukeboxPlayable {
    pub song: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AttributeModifiers {
    pub modifiers: Vec<Modifier>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Modifier {
    #[serde(rename = "type")]
    pub type_val: String,
    pub id: String,
    pub amount: f64,
    pub operation: Operation,
    // TODO: Make this an enum
    pub slot: String,
}

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct Food {
    pub nutrition: u32,
    pub saturation: f32,
    pub can_always_eat: Option<bool>,
}

#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct Equippable {
    pub allowed_entities: Option<ParameterValue>,
    pub camera_overlay: Option<String>,
    pub damage_on_hurt: bool,
    pub swappable: bool,
    pub slot: ArmorSlot,
    pub equip_sound: Option<String>,
    pub asset_id: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum ParameterValue {
    Primitive(String),
    List(Vec<String>),
}

#[derive(Deserialize, Clone, Copy, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum ArmorSlot {
    #[default]
    Head,
    Chest,
    Legs,
    Feet,

    Offhand,
    Body
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}
