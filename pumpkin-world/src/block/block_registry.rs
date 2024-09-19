use std::{collections::HashMap, sync::LazyLock};

use super::BlockState;
use crate::item::get_item_protocol_id;
use serde::Deserialize;

pub static BLOCKS: LazyLock<HashMap<String, RegistryBlockType>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../assets/blocks.json"))
        .expect("Could not parse block.json registry.")
});

pumpkin_macros::blocks_enum!();
pumpkin_macros::block_categories_enum!();

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

#[derive(Default, Copy, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct BlockId {
    pub data: u16,
}

impl BlockId {
    pub fn is_air(&self) -> bool {
        self.data == 0 || self.data == 12959 || self.data == 12958
    }

    pub fn get_id_mojang_repr(&self) -> i32 {
        self.data as i32
    }

    pub fn get_id(&self) -> u16 {
        self.data
    }

    pub fn get_as_item_id(&self) -> u32 {
        let id = BLOCKS
            .iter()
            .find(|(_, val)| val.states.iter().any(|state| state.id == *self))
            .map(|(key, _)| key.as_str())
            .unwrap();
        dbg!(id);
        get_item_protocol_id(id)
    }
}

impl From<BlockState> for BlockId {
    fn from(value: BlockState) -> Self {
        Self {
            data: value.get_id(),
        }
    }
}
