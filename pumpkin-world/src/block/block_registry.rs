use std::collections::HashMap;

use lazy_static::lazy_static;
use serde::Deserialize;

use crate::level::WorldError;

const BLOCKS_JSON: &str = include_str!("../../assets/blocks.json");

// 0 is air -> reasonable default
#[derive(Default, Deserialize, Debug, Hash, Clone, Copy, PartialEq, Eq)]
#[serde(transparent)]
pub struct BlockId {
    data: u16,
}

impl BlockId {
    pub fn new(
        block_id: &str,
        properties: Option<&HashMap<String, String>>,
    ) -> Result<Self, WorldError> {
        let mut block_states = BLOCKS
            .get(block_id)
            .ok_or(WorldError::BlockStateIdNotFound)?
            .states
            .iter();

        let block_state = match properties {
            Some(properties) => match block_states.find(|state| &state.properties == properties) {
                Some(state) => state,
                None => return Err(WorldError::BlockStateIdNotFound),
            },
            None => block_states
                .find(|state| state.is_default)
                .expect("Every Block should have at least 1 default state"),
        };

        Ok(block_state.id)
    }

    pub fn from_id(id: u16) -> Self {
        // TODO: add check if the id is actually valid
        Self { data: id }
    }

    pub fn is_air(&self) -> bool {
        self.data == 0
    }

    pub fn get_id(&self) -> u16 {
        self.data
    }

    /// An i32 is the way mojang internally represents their Blocks
    pub fn get_id_mojang_repr(&self) -> i32 {
        self.data as i32
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BlockDefinition {
    /// e.g. minecraft:door or minecraft:button
    #[serde(rename = "type")]
    category: String,

    /// Specifies the variant of the blocks category.
    /// e.g. minecraft:iron_door has the variant iron
    #[serde(rename = "block_set_type")]
    variant: Option<String>,
}

/// One possible state of a Block.
/// This could e.g. be an extended piston facing left.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BlockState {
    id: BlockId,

    /// Whether this is the default state of the Block
    #[serde(default, rename = "default")]
    is_default: bool,

    /// The propertise active for this `BlockState`.
    #[serde(default)]
    properties: HashMap<String, String>,
}

/// A fully-fledged block definition.
/// Stores the category, variant, all of the possible states and all of the possible properties.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BlockType {
    definition: BlockDefinition,
    states: Vec<BlockState>,

    // TODO is this safe to remove? It's currently not used in the Project. @lukas0008 @Snowiiii
    /// A list of valid property keys/values for a block.
    #[serde(default, rename = "properties")]
    valid_properties: HashMap<String, Vec<String>>,
}

lazy_static! {
    pub static ref BLOCKS: HashMap<String, BlockType> =
        serde_json::from_str(BLOCKS_JSON).expect("Could not parse block.json registry.");
}
