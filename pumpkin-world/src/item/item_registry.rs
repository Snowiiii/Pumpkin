use std::sync::LazyLock;

use serde::Deserialize;

const ITEMS_JSON: &str = include_str!("../../../assets/items.json");

pub static ITEMS: LazyLock<Vec<Item>> = LazyLock::new(|| {
    serde_json::from_str(ITEMS_JSON).expect("Could not parse items.json registry.")
});

pub fn get_item(name: &str) -> Option<&'static Item> {
    ITEMS.iter().find(|item| item.name == name)
}

pub fn get_item_by_id(id: u16) -> Option<&'static Item> {
    ITEMS.iter().find(|item| item.id == id)
}

#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct Item {
    pub id: u16,
    pub name: String,
    translation_key: String,
    pub max_stack: i8,
    max_durability: u16,
    break_sound: String,
    food: Option<FoodComponent>,
}

#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
struct FoodComponent {
    hunger: u16,
    saturation: f32,
    always_edible: bool,
    meat: bool,
    snack: bool,
}
