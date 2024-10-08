mod generator;
mod generic_generator;
mod implementation;
mod noise;
mod seed;

pub use generator::WorldGenerator;
use implementation::overworld::biome::plains::PlainsGenerator;
pub use seed::Seed;

use generator::GeneratorInit;

pub fn get_world_gen(seed: Seed) -> Box<dyn WorldGenerator> {
    // TODO decide which WorldGenerator to pick based on config.
    Box::new(PlainsGenerator::new(seed))
}
