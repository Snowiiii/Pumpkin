pub trait WindowPropertyTrait: Sized {
    fn to_id(self) -> i16 {
        0
    }
}

pub struct WindowProperty<T: WindowPropertyTrait> {
    window_property: T,
    value: i16
}

impl<T: WindowPropertyTrait> WindowProperty<T> {
    pub fn new(window_property: T,value: i16) -> Self {
        Self {
            window_property,
            value
        }
    }
    
    pub fn into_packet(self) -> (i16,i16) {
        (self.window_property.to_id(), self.value)
    }
}


pub enum Furnace {
    FireIcon,
    MaximumFuelBurnTime,
    ProgressArrow,
    MaximumProgress
}

impl WindowPropertyTrait for Furnace {
    fn to_id(self) -> i16 {
        self as i16
    }
}



pub enum EnchantmentTable {
    LevelRequirement{
        slot:u8
    },
    EnchantmentSeed,
    EnchantmentId{
        slot:u8
    },
    EnchantmentLevel{slot:u8},
}

impl WindowPropertyTrait for EnchantmentTable {
    fn to_id(self) -> i16 {
        use EnchantmentTable::*;

        (match self {
            LevelRequirement{slot} => slot,
            EnchantmentSeed => 3,
            EnchantmentId{slot} => 4+slot,
            EnchantmentLevel{slot} => 7+slot,
        }) as i16
    }
}

pub enum Beacon {
    PowerLevel,
    FirstPotionEffect,
    SecondPotionEffect
}

impl WindowPropertyTrait for Beacon {
    fn to_id(self) -> i16 {
        self as i16
    }
}

pub enum Anvil {
    RepairCost
}

impl WindowPropertyTrait for Anvil {}

pub enum BrewingStand{
    BrewTime,
    FuelTime
}

impl WindowPropertyTrait for BrewingStand {
    fn to_id(self) -> i16 {
        self as i16
    }
}

pub enum Stonecutter {
    SelectedRecipe
}

impl WindowPropertyTrait for Stonecutter {}

pub enum Loom {
    SelectedPattern
}

impl WindowPropertyTrait for Loom {}

pub enum Lectern {
    PageNumber
}

impl WindowPropertyTrait for Lectern {}

