use num_derive::{FromPrimitive, ToPrimitive};

pub mod container_click;
pub mod player;
pub mod window_property;

/// https://wiki.vg/Inventory
#[derive(Debug, ToPrimitive, FromPrimitive, Clone)]
pub enum WindowType {
    // not used
    Generic9x1,
    // not used
    Generic9x2,
    // General-purpose 3-row inventory. Used by Chest, minecart with chest, ender chest, and barrel
    Generic9x3,
    // not used
    Generic9x4,
    // not used
    Generic9x5,
    // Used by large chests
    Generic9x6,
    // General-purpose 3-by-3 square inventory, used by Dispenser and Dropper
    Generic3x3,
    // General-purpose 3-by-3 square inventory, used by the Crafter
    Craft3x3,
    Anvil,
    Beacon,
    BlastFurnace,
    BrewingStand,
    CraftingTable,
    EnchantmentTable,
    Furnace,
    Grindstone,
    // Hopper or minecart with hopper
    Hopper,
    Lectern,
    Loom,
    // Villager, Wandering Trader
    Merchant,
    ShulkerBox,
    SmithingTable,
    Smoker,
    CartographyTable,
    Stonecutter,
}

impl WindowType {
    pub const fn default_title(&self) -> &'static str {
        // TODO: Add titles here:
        /*match self {
            _ => "WINDOW TITLE",
        }*/
        "WINDOW TITLE"
    }
}

pub trait Container {
    const SLOTS: usize;

    fn item_mut(&mut self, slot: usize) -> &mut Item;
}

pub struct ContainerStruct<const SLOTS: usize> {
    slots: [Option<Item>; SLOTS],
    state_id: usize,
    open_by: Option<Vec<i32>>,
}

impl<const T: usize> ContainerStruct<T> {
    pub fn take_item(&mut self, slot: usize) -> Item {
        assert!(slot < T);
        let Some(item) = self.slots[slot] else {
            panic!()
        };
        self.slots[slot] = None;
        self.state_id += 1;
        item
    }
}
