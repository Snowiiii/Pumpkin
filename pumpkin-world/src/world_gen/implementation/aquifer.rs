use crate::{self as pumpkin_world, world_gen::noise::density::NoisePosImpl};
use enum_dispatch::enum_dispatch;
use pumpkin_macros::block_id;

use crate::block::BlockId;

#[derive(Clone)]
struct FluidLevel {
    y: i32,
    state: BlockId,
}

impl FluidLevel {
    fn get_block(&self, y: i32) -> BlockId {
        if y < self.y {
            self.state
        } else {
            block_id!("minecraft:air")
        }
    }
}

#[derive(Clone)]
pub struct FluidLevelSampler {
    sea_level: i32,
    fluid_level_1: FluidLevel,
    fluid_level_2: FluidLevel,
}

impl FluidLevelSampler {
    fn get_fluid_level(&self, _x: i32, y: i32, _z: i32) -> FluidLevel {
        if y < (-51).min(self.sea_level) {
            self.fluid_level_1.clone()
        } else {
            self.fluid_level_2.clone()
        }
    }
}

#[enum_dispatch(AquiferSamplerImpl)]
pub enum AquifierSampler {
    SeaLevel(AquiferSeaLevel),
}

#[enum_dispatch]
pub trait AquiferSamplerImpl {
    fn apply(&self, pos: &impl NoisePosImpl, density: f64) -> Option<BlockId>;
}

pub struct AquiferSeaLevel {
    level_sampler: FluidLevelSampler,
}

impl AquiferSeaLevel {
    pub fn new(level_sampler: FluidLevelSampler) -> Self {
        Self { level_sampler }
    }
}

impl AquiferSamplerImpl for AquiferSeaLevel {
    fn apply(&self, pos: &impl NoisePosImpl, density: f64) -> Option<BlockId> {
        if density > 0f64 {
            None
        } else {
            Some(
                self.level_sampler
                    .get_fluid_level(pos.x(), pos.y(), pos.z())
                    .get_block(pos.y()),
            )
        }
    }
}
