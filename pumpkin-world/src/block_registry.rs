use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::world::WorldError;

const BLOCKS_JSON: &str = include_str!("../blocks.json");

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
struct BlockDefinition {
    #[serde(rename = "type")]
    kind: String,
    block_set_type: Option<String>,
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
struct BlockState {
    default: Option<bool>,
    id: i64,
    properties: Option<HashMap<String, String>>,
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
struct BlocksElement {
    definition: BlockDefinition,
    properties: Option<HashMap<String, Vec<String>>>,
    states: Vec<BlockState>,
}

lazy_static! {
    static ref BLOCKS: HashMap<String, BlocksElement> =
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
            .find(|state| state.default.unwrap_or(false))
            .expect("Each block should have at least one default state")
            .id),
        Some(properties) => block
            .states
            .iter()
            .find(|state| state.properties.as_ref() == Some(properties))
            .map(|state| state.id)
            .ok_or(WorldError::BlockStateIdNotFound),
    };
    block_state_id
}
