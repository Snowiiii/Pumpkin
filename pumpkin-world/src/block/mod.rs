pub mod block_registry;
pub mod block_state;

use pumpkin_core::math::vector3::Vector3;

pub use block_state::BlockState;

pub enum BlockFace {
    Bottom = 0,
    Top,
    North,
    South,
    West,
    East,
}

pub struct InvalidBlockFace;

impl TryFrom<i32> for BlockFace {
    type Error = InvalidBlockFace;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Bottom),
            1 => Ok(Self::Top),
            2 => Ok(Self::North),
            3 => Ok(Self::South),
            4 => Ok(Self::West),
            5 => Ok(Self::East),
            _ => Err(InvalidBlockFace),
        }
    }
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
