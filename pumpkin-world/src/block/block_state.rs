use std::collections::HashMap;

use thiserror::Error;

use super::block_registry::{Block, BlockCategory, BLOCKS};

#[derive(Clone)]
pub struct BlockState {
    state_id: u16,
    block: Block,
    category: BlockCategory,
}

impl BlockState {
    pub const AIR: BlockState = BlockState {
        state_id: 0,
        block: Block::Air,
        category: BlockCategory::Air,
    };

    pub fn new(
        registry_id: &str,
        properties: Option<&HashMap<String, String>>,
    ) -> Result<Self, BlockStateError> {
        let block_registry = BLOCKS
            .get(registry_id)
            .ok_or(BlockStateError::BlockIdentifierNotFound)?;
        let mut block_states = block_registry.states.iter();

        let block_state = match properties {
            Some(properties) => block_states
                .find(|state| &state.properties == properties)
                .ok_or(BlockStateError::BlockStateIdNotFound)?,
            None => block_states
                .find(|state| state.is_default)
                .expect("Every Block should have at least 1 default state"),
        };

        Ok(Self {
            state_id: block_state.id.data,
            block: Block::from_registry_id(registry_id),
            category: BlockCategory::from_registry_id(&block_registry.definition.category),
        })
    }

    pub const fn new_unchecked(state_id: u16, block: Block, category: BlockCategory) -> Self {
        Self {
            state_id,
            block,
            category,
        }
    }

    pub fn is_air(&self) -> bool {
        self.category == BlockCategory::Air
    }

    pub fn get_id(&self) -> u16 {
        self.state_id
    }

    pub fn get_id_mojang_repr(&self) -> i32 {
        self.state_id as i32
    }

    pub fn of_block(&self, block: Block) -> bool {
        self.block == block
    }

    pub fn of_category(&self, category: BlockCategory) -> bool {
        self.category == category
    }
}

#[derive(Error, Debug)]
pub enum BlockStateError {
    #[error("The requested block identifier does not exist")]
    BlockIdentifierNotFound,
    #[error("The requested block state id does not exist")]
    BlockStateIdNotFound,
}
