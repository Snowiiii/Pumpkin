use std::{collections::HashMap, sync::LazyLock};

use itertools::Itertools;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
struct RegistryBlockDefinition {
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
#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
struct RegistryBlockState {
    pub id: i32,

    /// Whether this is the default state of the Block
    #[serde(default, rename = "default")]
    pub is_default: bool,

    /// The propertise active for this `BlockState`.
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

/// A fully-fledged block definition.
/// Stores the category, variant, all of the possible states and all of the possible properties.
#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
struct RegistryBlockType {
    pub definition: RegistryBlockDefinition,
    pub states: Vec<RegistryBlockState>,

    // TODO is this safe to remove? It's currently not used in the Project. @lukas0008 @Snowiiii
    /// A list of valid property keys/values for a block.
    #[serde(default, rename = "properties")]
    valid_properties: HashMap<String, Vec<String>>,
}

static BLOCKS: LazyLock<HashMap<String, RegistryBlockType>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/blocks.json"))
        .expect("Could not parse block.json registry.")
});

pub fn block_id_impl(item: TokenStream) -> TokenStream {
    let data = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
        .parse(item)
        .unwrap();
    let block_name = data
        .first()
        .expect("The first argument should be a block name");

    let block_name = match block_name {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(name) => name.value(),
            _ => panic!("The first argument should be a string"),
        },
        _ => panic!("The first argument should be a string"),
    };

    let mut properties = HashMap::new();
    for expr_thingy in data.into_iter().skip(1) {
        match expr_thingy {
            syn::Expr::Assign(assign) => {
                let left = match assign.left.as_ref() {
                    syn::Expr::Lit(lit) => match &lit.lit {
                        syn::Lit::Str(name) => name.value(),
                        _ => panic!(
                            "All not-first arguments should be assignments (\"foo\" = \"bar\")"
                        ),
                    },
                    _ => {
                        panic!("All not-first arguments should be assignments (\"foo\" = \"bar\")")
                    }
                };
                let right = match assign.right.as_ref() {
                    syn::Expr::Lit(lit) => match &lit.lit {
                        syn::Lit::Str(name) => name.value(),
                        _ => panic!(
                            "All not-first arguments should be assignments (\"foo\" = \"bar\")"
                        ),
                    },
                    _ => {
                        panic!("All not-first arguments should be assignments (\"foo\" = \"bar\")")
                    }
                };
                properties.insert(left, right);
            }
            _ => panic!("All not-first arguments should be assignments (\"foo\" = \"bar\")"),
        }
    }

    // panic!("{:?}", properties);

    let block_info = &BLOCKS
        .get(&block_name)
        .expect("Block with that name does not exist");

    let id = if properties.is_empty() {
        block_info
            .states
            .iter()
            .find(|state| state.is_default)
            .expect(
                "Error inside blocks.json file: Every Block should have at least 1 default state",
            )
            .id
    } else {
        match block_info
            .states
            .iter()
            .find(|state| state.properties == properties)
        {
            Some(state) => state.id,
            None => panic!(
                "Could not find block with these properties, the following are valid properties: \n{}",
                block_info
                    .valid_properties
                    .iter()
                    .map(|(name, values)| format!("{name} = {}", values.join(" | ")))
                    .join("\n")
            ),
        }
    };

    if std::env::var("CARGO_PKG_NAME").unwrap() == "pumpkin-world" {
        quote! {
          crate::block::block_id::BlockId::from_id(#id as u16)
        }
    } else {
        quote! {
          pumpkin_world::block::block_id::BlockId::from_id(#id as u16)
        }
    }
    .into()
}
