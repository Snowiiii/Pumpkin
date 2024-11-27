use std::{collections::HashMap, sync::LazyLock};

use quote::quote;
use serde::Deserialize;

static BLOCKS: LazyLock<TopLevel> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/blocks.json"))
        .expect("Could not parse blocks.json registry.")
});

static STATE_BY_REGISTRY_ID: LazyLock<HashMap<String, Block>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    for block in &BLOCKS.blocks {
        map.insert(block.name.clone(), block.clone());
    }
    map
});

pub(crate) fn block_state_impl(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_string = item.to_string();
    let registry_id = input_string.trim_matches('"');

    let default_state_id = STATE_BY_REGISTRY_ID
        .get(registry_id)
        .expect("Invalid registry id")
        .default_state_id;

    if std::env::var("CARGO_PKG_NAME").unwrap() == "pumpkin-world" {
        quote! {
            crate::block::BlockState {
                state_id: #default_state_id
          }
        }
        .into()
    } else {
        quote! {
          pumpkin_world::block::BlockState {
            state_id: #default_state_id
          }
        }
        .into()
    }
}

#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
struct TopLevel {
    block_entity_types: Vec<String>,
    shapes: Vec<Shape>,
    pub blocks: Vec<Block>,
}

#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
struct Block {
    pub id: u16,
    pub item_id: u16,
    pub hardness: f32,
    pub wall_variant_id: Option<u16>,
    pub translation_key: String,
    pub name: String,
    pub properties: Vec<Property>,
    pub default_state_id: u16,
    pub states: Vec<State>,
}
#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct Property {
    name: String,
    values: Vec<String>,
}
#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct State {
    pub id: u16,
    pub air: bool,
    pub luminance: u8,
    pub burnable: bool,
    pub opacity: Option<u32>,
    pub replaceable: bool,
    pub collision_shapes: Vec<u16>,
    pub block_entity_type: Option<u32>,
}
#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
struct Shape {
    min: [f32; 3],
    max: [f32; 3],
}
