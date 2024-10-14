use pumpkin_core::random::RandomDeriver;

use crate::world_gen::supplier::MultiNoiseSampler;

use super::router::NoiseRouter;

pub struct NoiseConfig<'a> {
    random_deriver: RandomDeriver,
    router: NoiseRouter<'a>,
}
