pub enum VeinType {
    Copper,
    Iron,
}

impl VeinType {
    pub fn min_y(&self) -> i32 {
        match self {
            Self::Copper => 0,
            Self::Iron => -60,
        }
    }

    pub fn max_y(&self) -> i32 {
        match self {
            Self::Copper => 50,
            Self::Iron => -8,
        }
    }

    pub fn overall_min_y() -> i32 {
        -60
    }

    pub fn overall_max_y() -> i32 {
        60
    }
}
