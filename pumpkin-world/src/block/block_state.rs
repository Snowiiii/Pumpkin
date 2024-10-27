use thiserror::Error;

use super::block_registry::get_block;

#[derive(Clone)]
pub struct BlockState {
    pub state_id: u16,
}

impl BlockState {
    pub const AIR: BlockState = BlockState { state_id: 0 };

    pub fn new(registry_id: &str) -> Result<Self, BlockStateError> {
        let block_registry =
            get_block(registry_id).ok_or(BlockStateError::BlockIdentifierNotFound)?;
        Ok(Self {
            state_id: block_registry.default_state_id,
        })
    }

    pub fn get_id(&self) -> u16 {
        self.state_id
    }

    pub fn get_id_mojang_repr(&self) -> i32 {
        self.state_id as i32
    }
}

#[derive(Error, Debug)]
pub enum BlockStateError {
    #[error("The requested block identifier does not exist")]
    BlockIdentifierNotFound,
    #[error("The requested block state id does not exist")]
    BlockStateIdNotFound,
}
