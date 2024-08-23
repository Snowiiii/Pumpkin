use num_derive::ToPrimitive;

pub mod player;
pub mod window_property;

/// https://wiki.vg/Inventory
#[derive(Debug, ToPrimitive, Clone)]
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
impl TryFrom<u8> for WindowType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(WindowType::Generic9x1),
            1 => Ok(WindowType::Generic9x2),
            2 => Ok(WindowType::Generic9x3),
            3 => Ok(WindowType::Generic9x4),
            4 => Ok(WindowType::Generic9x5),
            5 => Ok(WindowType::Generic9x6),
            6 => Ok(WindowType::Generic3x3),
            7 => Ok(WindowType::Craft3x3),
            8 => Ok(WindowType::Anvil),
            9 => Ok(WindowType::Beacon),
            10 => Ok(WindowType::BlastFurnace),
            11 => Ok(WindowType::BrewingStand),
            12 => Ok(WindowType::CraftingTable),
            13 => Ok(WindowType::EnchantmentTable),
            14 => Ok(WindowType::Furnace),
            15 => Ok(WindowType::Grindstone),
            16 => Ok(WindowType::Hopper),
            17 => Ok(WindowType::Lectern),
            18 => Ok(WindowType::Loom),
            19 => Ok(WindowType::Merchant),
            20 => Ok(WindowType::ShulkerBox),
            21 => Ok(WindowType::SmithingTable),
            22 => Ok(WindowType::Smoker),
            23 => Ok(WindowType::CartographyTable),
            24 => Ok(WindowType::Stonecutter),
            _ => Err(()),
        }
    }
}
