mod generator;
mod generic_generator;
mod seed;

pub use generator::WorldGenerator;
pub use seed::Seed;

#[allow(dead_code)]
pub fn get_world_gen(_: Seed) -> Box<dyn WorldGenerator> {
    todo!()
}
