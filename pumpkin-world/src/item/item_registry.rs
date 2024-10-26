use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

const ITEMS_JSON: &str = include_str!("../../../assets/items.json");

pub static ITEMS: LazyLock<HashMap<String, Item>> = LazyLock::new(|| {
    serde_json::from_str(ITEMS_JSON).expect("Could not parse items.json registry.")
});

#[derive(Deserialize, Clone, Debug)]
struct Item {
    id: u16,
    name: String,
    translation_key: String,
    max_stack: i8,
    max_durability: u16,
    break_sound: String,
    food: Option<FoodComponent>,
}

#[derive(Deserialize, Clone, Debug)]
struct FoodComponent {
    hunger: u16,
    saturation: f32,
    always_edible: bool,
    meat: bool,
    snack: bool,
}
