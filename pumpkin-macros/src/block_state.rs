use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

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

fn pascal_case(original: &str) -> String {
    let mut pascal = String::new();
    let mut capitalize = true;
    for ch in original.chars() {
        if ch == '_' {
            capitalize = true;
        } else if capitalize {
            pascal.push(ch.to_ascii_uppercase());
            capitalize = false;
        } else {
            pascal.push(ch);
        }
    }
    pascal
}

pub fn block_type_enum_impl() -> TokenStream {
    let categories: &HashSet<&str> = &BLOCKS
        .values()
        .map(|val| val.definition.category.as_str())
        .collect();

    let original_and_converted_stream = categories.iter().map(|key| {
        (
            key,
            pascal_case(key.split_once(':').expect("Bad minecraft id").1),
        )
    });
    let new_names: proc_macro2::TokenStream = original_and_converted_stream
        .clone()
        .map(|(_, x)| x)
        .join(",\n")
        .parse()
        .unwrap();

    let from_string: proc_macro2::TokenStream = original_and_converted_stream
        .clone()
        .map(|(original, converted)| format!("\"{}\" => BlockCategory::{},", original, converted))
        .join("\n")
        .parse()
        .unwrap();

    // I;ve never used macros before so call me out on this lol
    quote! {
        #[derive(PartialEq, Clone)]
        pub enum BlockCategory {
            #new_names
        }

        impl BlockCategory {
            pub fn from_registry_id(id: &str) -> BlockCategory {
                match id {
                    #from_string
                    _ => panic!("Not a valid block type id"),
                }
            }
        }
    }
    .into()
}

pub fn block_enum_impl() -> TokenStream {
    let original_and_converted_stream = &BLOCKS.keys().map(|key| {
        (
            key,
            pascal_case(key.split_once(':').expect("Bad minecraft id").1),
        )
    });
    let new_names: proc_macro2::TokenStream = original_and_converted_stream
        .clone()
        .map(|(_, x)| x)
        .join(",\n")
        .parse()
        .unwrap();

    let from_string: proc_macro2::TokenStream = original_and_converted_stream
        .clone()
        .map(|(original, converted)| format!("\"{}\" => Block::{},", original, converted))
        .join("\n")
        .parse()
        .unwrap();

    // I;ve never used macros before so call me out on this lol
    quote! {
        #[derive(PartialEq, Clone)]
        pub enum Block {
            #new_names
        }

        impl Block {
            pub fn from_registry_id(id: &str) -> Block {
                match id {
                    #from_string
                    _ => panic!("Not a valid block id"),
                }
            }
        }
    }
    .into()
}

pub fn block_state_impl(item: TokenStream) -> TokenStream {
    let data = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
        .parse(item)
        .unwrap();
    let block_name = data
        .first()
        .expect("The first argument should be a block name");

    let block_name = match block_name {
        syn::Expr::Lit(lit) => match &lit.lit {
            syn::Lit::Str(name) => name.value(),
            _ => panic!("The first argument should be a string, have: {:?}", lit),
        },
        _ => panic!(
            "The first argument should be a string, have: {:?}",
            block_name
        ),
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

    let state = if properties.is_empty() {
        block_info
            .states
            .iter()
            .find(|state| state.is_default)
            .expect(
                "Error inside blocks.json file: Every Block should have at least 1 default state",
            )
    } else {
        match block_info
            .states
            .iter()
            .find(|state| state.properties == properties)
        {
            Some(state) => state,
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

    let id = state.id;
    if std::env::var("CARGO_PKG_NAME").unwrap() == "pumpkin-world" {
        let category_name: proc_macro2::TokenStream = format!(
            "crate::block::BlockCategory::{}",
            &pascal_case(
                &block_info
                    .definition
                    .category
                    .split_once(':')
                    .expect("Bad minecraft id")
                    .1,
            )
        )
        .parse()
        .unwrap();
        let block_name: proc_macro2::TokenStream = format!(
            "crate::block::Block::{}",
            &pascal_case(&block_name.split_once(':').expect("Bad minecraft id").1,)
        )
        .parse()
        .unwrap();
        quote! {
            crate::block::BlockState {
                state_id: #id as u16,
                category: #category_name,
                block: #block_name,
          }
        }
        .into()
    } else {
        let category_name: proc_macro2::TokenStream = format!(
            "pumpkin_world::block::BlockCategory::{}",
            &pascal_case(
                &block_info
                    .definition
                    .category
                    .split_once(':')
                    .expect("Bad minecraft id")
                    .1,
            )
        )
        .parse()
        .unwrap();
        let block_name: proc_macro2::TokenStream = format!(
            "pumpkin_world::block::Block::{}",
            &pascal_case(&block_name.split_once(':').expect("Bad minecraft id").1,)
        )
        .parse()
        .unwrap();
        quote! {
          pumpkin_world::block::BlockState {
                state_id: #id as u16,
                category: #category_name,
                block: #block_name,
          }
        }
        .into()
    }
}
