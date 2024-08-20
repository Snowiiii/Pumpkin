mod item_registry;
mod item_categories;
pub use item_registry::ITEMS;
#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
/// Item Rarity
pub enum Rarity {
    Common,
    UnCommon,
    Rare,
    Epic,
}

#[derive(Clone, Copy)]
pub struct Item {
    pub item_count: u32,
    // This ID is the numerical protocol ID, not the usual minecraft::block ID.
    pub item_id: u32,
    // TODO: Add Item Components
}
