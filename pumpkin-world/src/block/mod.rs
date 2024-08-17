use num_derive::FromPrimitive;

pub mod block_registry;

#[derive(FromPrimitive)]
pub enum BlockFace {
    Bottom = 0,
    Top,
    North,
    South,
    West,
    East,
}
