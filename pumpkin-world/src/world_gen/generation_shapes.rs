use super::{biome_coords::to_block, height_limit::HeightLimitViewImpl};

pub struct GenerationShape {
    min_y: i8,
    height: u16,
    /// Max: 4
    horizontal_size: u8,
    /// Max: 4
    vertical_size: u8,
}

impl GenerationShape {
    pub const SURFACE: Self = Self {
        min_y: -64,
        height: 384,
        horizontal_size: 1,
        vertical_size: 2,
    };
    pub const NETHER: Self = Self {
        min_y: 0,
        height: 128,
        horizontal_size: 1,
        vertical_size: 2,
    };
    pub const END: Self = Self {
        min_y: 0,
        height: 128,
        horizontal_size: 2,
        vertical_size: 1,
    };
    pub const CAVES: Self = Self {
        min_y: -64,
        height: 192,
        horizontal_size: 1,
        vertical_size: 2,
    };
    pub const FLOATING_ISLANDS: Self = Self {
        min_y: 0,
        height: 256,
        horizontal_size: 2,
        vertical_size: 1,
    };

    pub fn vertical_cell_block_count(&self) -> u8 {
        to_block(self.vertical_size)
    }

    pub fn horizontal_cell_block_count(&self) -> u8 {
        to_block(self.horizontal_size)
    }

    pub fn min_y(&self) -> i8 {
        self.min_y
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn max_y(&self) -> u16 {
        if self.min_y >= 0 {
            self.height + self.min_y as u16
        } else {
            self.height - self.min_y.unsigned_abs() as u16
        }
    }

    pub fn trim_height(&self, limit: &dyn HeightLimitViewImpl) -> Self {
        let new_min = self.min_y.max(limit.bottom_y());

        let this_top = if self.min_y >= 0 {
            self.height + self.min_y as u16
        } else {
            self.height - self.min_y.unsigned_abs() as u16
        };

        let new_top = this_top.min(limit.top_y());

        let new_height = if new_min >= 0 {
            new_top - new_min as u16
        } else {
            new_top + new_min.unsigned_abs() as u16
        };

        Self {
            min_y: new_min,
            height: new_height,
            horizontal_size: self.horizontal_size,
            vertical_size: self.vertical_size,
        }
    }
}
