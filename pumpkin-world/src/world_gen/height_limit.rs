use enum_dispatch::enum_dispatch;

use super::section_coords;

#[enum_dispatch]
pub enum HeightLimitView {
    Standard(StandardHeightLimitView),
}

#[enum_dispatch(HeightLimitView)]
pub trait HeightLimitViewImpl {
    fn height(&self) -> u16;

    fn bottom_y(&self) -> i8;

    fn top_y(&self) -> u16 {
        if self.bottom_y() >= 0 {
            self.height() + self.bottom_y() as u16
        } else {
            self.height() - self.bottom_y().unsigned_abs() as u16
        }
    }

    fn vertical_section_count(&self) -> u16 {
        let bottom_section = self.bottom_section_coord();
        if bottom_section >= 0 {
            self.top_section_coord() - self.bottom_section_coord() as u16
        } else {
            self.top_section_coord() + self.bottom_section_coord().unsigned_abs() as u16
        }
    }

    fn bottom_section_coord(&self) -> i8 {
        section_coords::block_to_section(self.bottom_y())
    }

    fn top_section_coord(&self) -> u16 {
        section_coords::block_to_section(self.top_y() - 1) + 1
    }

    fn out_of_height(&self, height: i16) -> bool {
        height < self.bottom_y() as i16 || height as i32 >= self.top_y() as i32
    }

    fn section_index(&self, y: i32) -> usize {
        self.section_coord_to_index(section_coords::block_to_section(y))
    }

    fn section_coord_to_index(&self, coord: i32) -> usize {
        (coord - self.bottom_section_coord() as i32) as usize
    }

    fn section_index_to_coord(&self, index: usize) -> i32 {
        index as i32 + self.bottom_section_coord() as i32
    }
}

pub struct StandardHeightLimitView {
    height: u16,
    bottom_y: i8,
}

impl StandardHeightLimitView {
    pub fn new(height: u16, bottom_y: i8) -> Self {
        Self { height, bottom_y }
    }
}

impl HeightLimitViewImpl for StandardHeightLimitView {
    fn height(&self) -> u16 {
        self.height
    }

    fn bottom_y(&self) -> i8 {
        self.bottom_y
    }
}
