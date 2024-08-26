mod item_categories;
mod item_registry;
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

#[derive(Clone, Copy, Debug)]
pub struct ItemStack {
    pub item_count: u8,
    // This ID is the numerical protocol ID, not the usual minecraft::block ID.
    pub item_id: u32,
    // TODO: Add Item Components
}
