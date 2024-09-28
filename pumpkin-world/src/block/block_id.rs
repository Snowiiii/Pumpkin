use std::collections::HashMap;

use serde::Deserialize;

use super::block_registry::BLOCKS;
use crate::level::WorldError;

// 0 is air -> reasonable default
#[derive(Default, Deserialize, Debug, Hash, Clone, Copy, PartialEq, Eq)]
#[serde(transparent)]
pub struct BlockId {
    data: u16,
}

impl BlockId {
    pub const AIR: Self = Self::from_id(0);

    pub fn new(
        text_id: &str,
        properties: Option<&HashMap<String, String>>,
    ) -> Result<Self, WorldError> {
        let mut block_states = BLOCKS
            .get(text_id)
            .ok_or(WorldError::BlockIdentifierNotFound)?
            .states
            .iter();

        let block_state = match properties {
            Some(properties) => block_states
                .find(|state| &state.properties == properties)
                .ok_or_else(|| WorldError::BlockStateIdNotFound)?,
            None => block_states
                .find(|state| state.is_default)
                .expect("Every Block should have at least 1 default state"),
        };

        Ok(block_state.id)
    }

    pub const fn from_id(id: u16) -> Self {
        // TODO: add check if the id is actually valid
        Self { data: id }
    }

    pub fn is_air(&self) -> bool {
        self.data == 0 || self.data == 12959 || self.data == 12958
    }

    pub fn get_id(&self) -> u16 {
        self.data
    }

    /// An i32 is the way mojang internally represents their Blocks
    pub fn get_id_mojang_repr(&self) -> i32 {
        self.data as i32
    }
}
