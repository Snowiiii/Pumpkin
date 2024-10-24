use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;

use super::{BlockState, BlockString};

pub static BLOCKS: LazyLock<HashMap<String, RegistryBlockType>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../assets/blocks.json"))
        .expect("Could not parse block.json registry.")
});

pub static BLOCK_IDS_TO_BLOCK_STRING: LazyLock<HashMap<BlockId, BlockString>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        for (block_name, registry_block) in &*BLOCKS {
            for state in registry_block.states.iter() {
                let block_string = BlockString {
                    name: block_name.as_str(),
                    properties: &state.properties,
                };
                map.insert(state.id, block_string);
            }
        }
        map
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

    /// This is advised to use only when necessary, since most of the time, you can get away with compairing block id's, which is much cheaper.
    pub fn get_block_string(&self) -> Option<&'static BlockString> {
        BLOCK_IDS_TO_BLOCK_STRING.get(self)
    }
}

impl From<BlockState> for BlockId {
    fn from(value: BlockState) -> Self {
        Self {
            data: value.get_id(),
        }
    }
}
