use pumpkin_core::math::vector3::Vector3;

use crate::block::block_state::BlockState;

pub struct ProtoChunk {
    // may want to use chunk status
}

impl ProtoChunk {
    pub fn get_block_state(&self, _pos: &Vector3<i32>) -> BlockState {
        unimplemented!()
    }
}
