use num_derive::ToPrimitive;
use num_traits::ToPrimitive;

pub trait WindowPropertyTrait {
    fn to_id(self) -> i16;
}

impl<T: ToPrimitive> WindowPropertyTrait for T {
    fn to_id(self) -> i16 {
        self.to_i16().unwrap()
    }
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
#[derive(ToPrimitive)]
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
#[derive(ToPrimitive)]
pub enum Beacon {
    PowerLevel,
    FirstPotionEffect,
    SecondPotionEffect,
}

#[derive(ToPrimitive)]
pub enum Anvil {
    RepairCost,
}

#[derive(ToPrimitive)]
pub enum BrewingStand {
    BrewTime,
    FuelTime,
}

#[derive(ToPrimitive)]
pub enum Stonecutter {
    SelectedRecipe,
}

#[derive(ToPrimitive)]
pub enum Loom {
    SelectedPattern,
}

#[derive(ToPrimitive)]
pub enum Lectern {
    PageNumber,
}
