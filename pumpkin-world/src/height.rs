use enum_dispatch::enum_dispatch;

use crate::world_gen::chunk::Chunk;

#[enum_dispatch]
pub enum HeightLimitView {
    Standard(StandardHeightLimitView),
    Chunk(Chunk),
}

#[enum_dispatch(HeightLimitView)]
pub trait HeightLimitViewImpl {
    fn height(&self) -> i32;

    fn bottom_y(&self) -> i32;

    fn top_y(&self) -> i32 {
        self.bottom_y() + self.height()
    }

    fn vertical_section_count(&self) -> i32 {
        self.top_section_coord() - self.bottom_section_coord()
    }

    fn bottom_section_coord(&self) -> i32 {
        self.bottom_y() >> 4
    }

    fn top_section_coord(&self) -> i32 {
        ((self.top_y() - 1) >> 4) + 1
    }

    fn out_of_height(&self, height: i32) -> bool {
        height < self.bottom_y() || height >= self.top_y()
    }

    fn section_index(&self, y: i32) -> i32 {
        self.section_coord_to_index(y >> 4)
    }

    fn section_coord_to_index(&self, coord: i32) -> i32 {
        coord - self.bottom_section_coord()
    }

    fn section_index_to_coord(&self, index: i32) -> i32 {
        index + self.bottom_section_coord()
    }
}

pub struct StandardHeightLimitView {
    height: i32,
    bottom_y: i32,
}

impl StandardHeightLimitView {
    pub fn new(height: i32, bottom_y: i32) -> Self {
        Self { height, bottom_y }
    }
}

impl HeightLimitViewImpl for StandardHeightLimitView {
    fn height(&self) -> i32 {
        self.height
    }

    fn bottom_y(&self) -> i32 {
        self.bottom_y
    }
}
