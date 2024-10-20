mod item_categories;
pub mod item_registry;
pub use item_registry::{get_item_protocol_id, ITEMS};
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
    pub item_id: u16,
    // TODO: Add Item Components
}

impl PartialEq for ItemStack {
    fn eq(&self, other: &Self) -> bool {
        self.item_id == other.item_id
    }
}

impl ItemStack {
    pub fn new(item_count: u8, item_id: u16) -> Self {
        Self {
            item_count,
            item_id,
        }
    }
}
