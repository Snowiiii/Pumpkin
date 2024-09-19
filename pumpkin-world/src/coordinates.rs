use std::ops::Deref;

use crate::{WORLD_LOWEST_Y, WORLD_MAX_Y};
use derive_more::derive::{AsMut, AsRef, Display, Into};
use num_traits::{PrimInt, Signed, Unsigned};
use pumpkin_core::math::position::WorldPosition;
use pumpkin_core::math::vector2::Vector2;
use pumpkin_core::math::vector3::Vector3;
use serde::{Deserialize, Serialize};

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, AsRef, AsMut, Into, Display,
)]
#[serde(transparent)]
pub struct Height(pub i16);

impl Height {
    pub fn from_absolute(height: u16) -> Self {
        (height as i16 - WORLD_LOWEST_Y.abs()).into()
    }

    /// Absolute height ranges from `0..WORLD_HEIGHT`
    /// instead of `WORLD_LOWEST_Y..WORLD_MAX_Y`
    pub fn get_absolute(self) -> u16 {
        (self.0 + WORLD_LOWEST_Y.abs()) as u16
    }
}

impl<T: PrimInt + Signed> From<T> for Height {
    fn from(height: T) -> Self {
        let height = height.to_i16().unwrap();

        assert!(height <= WORLD_MAX_Y);
        assert!(height >= WORLD_LOWEST_Y);
        Self(height)
    }
}

impl Deref for Height {
    type Target = i16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, AsRef, AsMut, Into, Display,
)]
#[repr(transparent)]
pub struct ChunkRelativeOffset(u8);

impl<T: PrimInt + Unsigned> From<T> for ChunkRelativeOffset {
    fn from(scalar: T) -> Self {
        let scalar = scalar.to_u8().unwrap();

        assert!(scalar < 16);
        Self(scalar)
    }
}

impl Deref for ChunkRelativeOffset {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
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
    pub x: ChunkRelativeOffset,
    pub y: Height,
    pub z: ChunkRelativeOffset,
}

impl ChunkRelativeBlockCoordinates {
    pub fn with_chunk_coordinates(self, chunk_coordinates: Vector2<i32>) -> BlockCoordinates {
        BlockCoordinates {
            x: *self.x as i32 + chunk_coordinates.x * 16,
            y: self.y,
            z: *self.z as i32 + chunk_coordinates.z * 16,
        }
    }
}

impl From<WorldPosition> for ChunkRelativeBlockCoordinates {
    fn from(pos: WorldPosition) -> Self {
        Self {
            x: ChunkRelativeOffset((pos.0.x / 16) as u8),
            y: Height(pos.0.y as i16),
            z: ChunkRelativeOffset((pos.0.z / 16) as u8),
        }
    }
}

impl From<Vector3<f64>> for ChunkRelativeBlockCoordinates {
    fn from(pos: Vector3<f64>) -> Self {
        Self {
            x: ChunkRelativeOffset((pos.x / 16.).round() as u8),
            y: Height(pos.y.ceil() as i16),
            z: ChunkRelativeOffset((pos.z / 16.).round() as u8),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkRelativeXZBlockCoordinates {
    pub x: ChunkRelativeOffset,
    pub z: ChunkRelativeOffset,
}

impl ChunkRelativeXZBlockCoordinates {
    pub fn with_chunk_coordinates(&self, chunk_coordinates: Vector2<i32>) -> XZBlockCoordinates {
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

impl From<Vector3<i32>> for ChunkRelativeBlockCoordinates {
    fn from(value: Vector3<i32>) -> Self {
        Self {
            x: (value.x as u8).into(),
            z: (value.z as u8).into(),
            y: value.y.into(),
        }
    }
}
