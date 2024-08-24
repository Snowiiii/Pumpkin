use std::ops::Deref;

use crate::{WORLD_LOWEST_Y, WORLD_MAX_Y};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Height {
    height: i16,
}

impl Height {
    pub fn new(height: i16) -> Self {
        assert!(height <= WORLD_MAX_Y);
        assert!(height >= WORLD_LOWEST_Y);

        Self { height }
    }

    pub fn from_absolute(height: u16) -> Self {
        Self::new(height as i16 - WORLD_LOWEST_Y.abs())
    }

    /// Absolute height ranges from `0..WORLD_HEIGHT`
    /// instead of `WORLD_LOWEST_Y..WORLD_MAX_Y`
    pub fn get_absolute(&self) -> u16 {
        (self.height + WORLD_LOWEST_Y.abs()) as u16
    }
}

impl Deref for Height {
    type Target = i16;

    fn deref(&self) -> &Self::Target {
        &self.height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkRelativeScalar {
    scalar: u8,
}

macro_rules! derive_chunk_relative_scalar_from_int_impl {
    ($integer:ty) => {
        impl From<$integer> for ChunkRelativeScalar {
            fn from(scalar: $integer) -> Self {
                assert!(scalar < 16);
                Self {
                    scalar: scalar as u8,
                }
            }
        }
    };
}

derive_chunk_relative_scalar_from_int_impl! {u8}
derive_chunk_relative_scalar_from_int_impl! {u16}
derive_chunk_relative_scalar_from_int_impl! {u32}
derive_chunk_relative_scalar_from_int_impl! {u64}
derive_chunk_relative_scalar_from_int_impl! {u128}
derive_chunk_relative_scalar_from_int_impl! {usize}

impl Deref for ChunkRelativeScalar {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.scalar
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockCoordinates {
    pub x: i32,
    pub y: Height,
    pub z: i32,
}

/// BlockCoordinates that do not specify a height.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XZBlockCoordinates {
    pub x: i32,
    pub z: i32,
}

/// Coordinates of a block relative to a chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkRelativeBlockCoordinates {
    pub x: ChunkRelativeScalar,
    pub y: Height,
    pub z: ChunkRelativeScalar,
}

impl ChunkRelativeBlockCoordinates {
    pub fn with_chunk_coordinates(&self, chunk_coordinates: ChunkCoordinates) -> BlockCoordinates {
        BlockCoordinates {
            x: *self.x as i32 + chunk_coordinates.x * 16,
            y: self.y,
            z: *self.z as i32 + chunk_coordinates.z * 16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkRelativeXZBlockCoordinates {
    pub x: ChunkRelativeScalar,
    pub z: ChunkRelativeScalar,
}

impl ChunkRelativeXZBlockCoordinates {
    pub fn with_chunk_coordinates(
        &self,
        chunk_coordinates: ChunkCoordinates,
    ) -> XZBlockCoordinates {
        XZBlockCoordinates {
            x: *self.x as i32 + chunk_coordinates.x * 16,
            z: *self.z as i32 + chunk_coordinates.z * 16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkCoordinates {
    pub x: i32,
    pub z: i32,
}

macro_rules! impl_get_absolute_height {
    ($struct_name:ident) => {
        impl $struct_name {}
    };
}

impl_get_absolute_height! {BlockCoordinates}
impl_get_absolute_height! {ChunkRelativeBlockCoordinates}
