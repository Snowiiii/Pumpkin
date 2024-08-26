use std::ops::Deref;

use num_traits::{PrimInt, Signed, Unsigned};
use serde::{Deserialize, Serialize};

use crate::{WORLD_LOWEST_Y, WORLD_MAX_Y};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(transparent)]
pub struct Height {
    height: i16,
}

impl Height {
    pub fn from_absolute(height: u16) -> Self {
        (height as i16 - WORLD_LOWEST_Y.abs()).into()
    }

    /// Absolute height ranges from `0..WORLD_HEIGHT`
    /// instead of `WORLD_LOWEST_Y..WORLD_MAX_Y`
    pub fn get_absolute(self) -> u16 {
        (self.height + WORLD_LOWEST_Y.abs()) as u16
    }
}

impl<T: PrimInt + Signed> From<T> for Height {
    fn from(height: T) -> Self {
        let height = height.to_i16().unwrap();

        assert!(height <= WORLD_MAX_Y);
        assert!(height >= WORLD_LOWEST_Y);
        Self { height }
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

impl<T: PrimInt + Unsigned> From<T> for ChunkRelativeScalar {
    fn from(scalar: T) -> Self {
        let scalar = scalar.to_u8().unwrap();

        assert!(scalar < 16);
        Self { scalar }
    }
}

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

impl XZBlockCoordinates {
    pub fn with_y(self, height: Height) -> BlockCoordinates {
        BlockCoordinates {
            x: self.x,
            y: height,
            z: self.z,
        }
    }
}

/// Coordinates of a block relative to a chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkRelativeBlockCoordinates {
    pub x: ChunkRelativeScalar,
    pub y: Height,
    pub z: ChunkRelativeScalar,
}

impl ChunkRelativeBlockCoordinates {
    pub fn with_chunk_coordinates(self, chunk_coordinates: ChunkCoordinates) -> BlockCoordinates {
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

    pub fn with_y(self, height: Height) -> ChunkRelativeBlockCoordinates {
        ChunkRelativeBlockCoordinates {
            x: self.x,
            y: height,
            z: self.z,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkCoordinates {
    pub x: i32,
    pub z: i32,
}
