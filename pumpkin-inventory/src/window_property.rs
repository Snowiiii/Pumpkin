pub trait WindowPropertyTrait {
    fn to_id(self) -> i16;
}

pub struct WindowProperty<T: WindowPropertyTrait> {
    window_property: T,
    value: i16,
}

impl<T: WindowPropertyTrait> WindowProperty<T> {
    pub fn new(window_property: T, value: i16) -> Self {
        Self {
            window_property,
            value,
        }
    }

    pub fn into_tuple(self) -> (i16, i16) {
        (self.window_property.to_id(), self.value)
    }
}
#[repr(u8)]
pub enum Furnace {
    FireIcon,
    MaximumFuelBurnTime,
    ProgressArrow,
    MaximumProgress,
}

pub enum EnchantmentTable {
    LevelRequirement { slot: u8 },
    EnchantmentSeed,
    EnchantmentId { slot: u8 },
    EnchantmentLevel { slot: u8 },
}

impl WindowPropertyTrait for EnchantmentTable {
    fn to_id(self) -> i16 {
        use EnchantmentTable::*;

        (match self {
            LevelRequirement { slot } => slot,
            EnchantmentSeed => 3,
            EnchantmentId { slot } => 4 + slot,
            EnchantmentLevel { slot } => 7 + slot,
        }) as i16
    }
}
#[repr(u8)]
pub enum Beacon {
    PowerLevel,
    FirstPotionEffect,
    SecondPotionEffect,
}

#[repr(u8)]
pub enum Anvil {
    RepairCost,
}

#[repr(u8)]
pub enum BrewingStand {
    BrewTime,
    FuelTime,
}

#[repr(u8)]
pub enum Stonecutter {
    SelectedRecipe,
}

#[repr(u8)]
pub enum Loom {
    SelectedPattern,
}

#[repr(u8)]
pub enum Lectern {
    PageNumber,
}
