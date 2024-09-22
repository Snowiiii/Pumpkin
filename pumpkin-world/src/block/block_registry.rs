use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

use super::block_id::BlockId;

pub static BLOCKS: LazyLock<HashMap<String, RegistryBlockType>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../assets/blocks.json"))
        .expect("Could not parse block.json registry.")
});

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RegistryBlockDefinition {
    /// e.g. minecraft:door or minecraft:button
    #[serde(rename = "type")]
    pub category: String,

    /// Specifies the variant of the blocks category.
    /// e.g. minecraft:iron_door has the variant iron
    #[serde(rename = "block_set_type")]
    pub variant: Option<String>,
}

/// One possible state of a Block.
/// This could e.g. be an extended piston facing left.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RegistryBlockState {
    pub id: BlockId,

    /// Whether this is the default state of the Block
    #[serde(default, rename = "default")]
    pub is_default: bool,

    /// The propertise active for this `BlockState`.
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

/// A fully-fledged block definition.
/// Stores the category, variant, all of the possible states and all of the possible properties.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RegistryBlockType {
    pub definition: RegistryBlockDefinition,
    pub states: Vec<RegistryBlockState>,

    // TODO is this safe to remove? It's currently not used in the Project. @lukas0008 @Snowiiii
    /// A list of valid property keys/values for a block.
    #[serde(default, rename = "properties")]
    valid_properties: HashMap<String, Vec<String>>,
}
