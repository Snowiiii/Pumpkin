#![allow(dead_code)]

mod blender;
pub mod chunk_noise;
pub mod generation_shapes;
mod generator;
mod generic_generator;
pub mod height_limit;
mod implementation;
pub mod noise;
mod positions;
pub mod proto_chunk;
pub mod sampler;
mod seed;

pub use generator::WorldGenerator;
use implementation::{
    //overworld::biome::plains::PlainsGenerator,
    test::{TestBiomeGenerator, TestGenerator, TestTerrainGenerator},
};
pub use seed::Seed;

use generator::GeneratorInit;

pub fn get_world_gen(seed: Seed) -> Box<dyn WorldGenerator> {
    // TODO decide which WorldGenerator to pick based on config.
    //Box::new(PlainsGenerator::new(seed))
    Box::new(TestGenerator::<TestBiomeGenerator, TestTerrainGenerator>::new(seed))
}

pub mod section_coords {
    use num_traits::PrimInt;

    #[inline]
    pub fn block_to_section<T>(coord: T) -> T
    where
        T: PrimInt,
    {
        coord >> 4
    }

    #[inline]
    pub fn section_to_block<T>(coord: T) -> T
    where
        T: PrimInt,
    {
        coord << 4
    }
}

pub mod biome_coords {
    use num_traits::PrimInt;

    #[inline]
    pub fn from_block<T>(coord: T) -> T
    where
        T: PrimInt,
    {
        coord >> 2
    }

    #[inline]
    pub fn to_block<T>(coord: T) -> T
    where
        T: PrimInt,
    {
        coord << 2
    }

    #[inline]
    pub fn from_chunk<T>(coord: T) -> T
    where
        T: PrimInt,
    {
        coord << 2
    }

    #[inline]
    pub fn to_chunk<T>(coord: T) -> T
    where
        T: PrimInt,
    {
        coord >> 2
    }
}

#[derive(PartialEq)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}
