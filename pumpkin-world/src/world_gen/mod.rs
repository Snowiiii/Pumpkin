mod generator;
mod generic_generator;
mod implementations;
mod seed;

pub use generator::WorldGenerator;
pub use seed::Seed;

use generator::GeneratorInit;
use implementations::superflat::SuperflatGenerator;

#[allow(dead_code)]
pub fn get_world_gen(seed: Seed) -> Box<dyn WorldGenerator> {
    // TODO decide which WorldGenerator to pick based on config.
    Box::new(SuperflatGenerator::new(seed))
}
