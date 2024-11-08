#![allow(dead_code)]

mod blender;
mod generator;
mod generic_generator;
pub mod height_limit;
mod implementation;
mod noise;
mod positions;
mod proto_chunk;
mod sampler;
mod seed;

pub use generator::WorldGenerator;
use implementation::{
    custom::CustomGenerator, overworld::biome::plains::PlainsGenerator,
    superflat::SuperflatGenerator, void::VoidGenerator,
};
use pumpkin_config::{GeneratorType, BASIC_CONFIG};
pub use seed::Seed;

use generator::GeneratorInit;

pub fn get_world_gen(seed: Seed) -> Box<dyn WorldGenerator> {
    match &BASIC_CONFIG.generator {
        GeneratorType::Simple(name) => match name.as_str() {
            "Void" => Box::new(VoidGenerator::new()),
            "Superflat" => Box::new(SuperflatGenerator::new()),
            "BeautifulPlains" => Box::new(PlainsGenerator::new(seed)),
            _ => panic!("unknown generator"),
        },
        GeneratorType::WithLayers(biom, layers) => {
            Box::new(CustomGenerator::new(biom.clone(), layers))
        }
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
