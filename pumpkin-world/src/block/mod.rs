use num_derive::FromPrimitive;

pub mod block_id;
mod block_registry;

pub use block_id::BlockId;
use pumpkin_core::math::vector3::Vector3;

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
            BlockFace::East => (1, 0, 0),
            BlockFace::North => (0, 0, -1),
            BlockFace::South => (0, 0, 1),
            BlockFace::Top => (0, 1, 0),
            BlockFace::West => (-1, 0, 0),
        }
        .into()
    }
}
