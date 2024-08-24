use std::collections::HashMap;

use lazy_static::lazy_static;
use serde::Deserialize;

use crate::level::WorldError;

const BLOCKS_JSON: &str = include_str!("../../assets/blocks.json");

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
    id: i64,

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

pub fn block_id_and_properties_to_block_state_id(
    block_id: &str,
    properties: Option<&HashMap<String, String>>,
) -> Result<i64, WorldError> {
    let block = match BLOCKS.get(block_id) {
        Some(block) => block,
        None => return Err(WorldError::BlockStateIdNotFound),
    };
    let block_state_id = match properties {
        None => Ok(block
            .states
            .iter()
            .find(|state| state.is_default)
            .expect("Each block should have at least one default state")
            .id),
        Some(properties) => block
            .states
            .iter()
            .find(|state| &state.properties == properties)
            .map(|state| state.id)
            .ok_or(WorldError::BlockStateIdNotFound),
    };
    block_state_id
}
