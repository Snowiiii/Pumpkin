use num_derive::FromPrimitive;

pub mod block_registry;
pub mod block_state;
mod block_string;

use pumpkin_core::math::vector3::Vector3;

pub use block_registry::BlockId;
pub use block_state::BlockState;
pub use block_string::BlockString;

#[derive(FromPrimitive)]
pub enum BlockFace {
    Bottom = 0,
    Top,
    North,
    South,
    West,
    East,
}

impl BlockFace {
    pub fn to_offset(&self) -> Vector3<i32> {
        match self {
            BlockFace::Bottom => (0, -1, 0),
            BlockFace::Top => (0, 1, 0),
            BlockFace::North => (0, 0, -1),
            BlockFace::South => (0, 0, 1),
            BlockFace::West => (-1, 0, 0),
            BlockFace::East => (1, 0, 0),
        }
        .into()
    }
}
