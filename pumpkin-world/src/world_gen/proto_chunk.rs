use pumpkin_core::math::vector3::Vector3;

use crate::block::block_state::BlockState;

pub struct ProtoChunk {
    state: GenerationState,
}

impl ProtoChunk {
    pub fn get_block_state(&self, _pos: &Vector3<i32>) -> BlockState {
        unimplemented!()
    }
}

#[derive(Clone, PartialEq, PartialOrd)]
pub enum GenerationState {
    Empty,
    StructureStart,
    StructureRef,
    Biome,
    Noise,
    Surface,
    Carver,
    Feature,
    InitLight,
    Light,
    Spawn,
    Full,
}
