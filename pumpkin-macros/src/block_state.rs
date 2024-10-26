use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use itertools::Itertools;
use proc_macro::TokenStream;
use quote::quote;
use serde::Deserialize;
use syn::parse::Parser;

#[derive(Deserialize, Clone, Debug)]
struct TopLevel {
    blocks: Vec<Block>,
    shapes: Vec<Shape>,
    block_entity_types: Vec<BlockEntityKind>,
}

#[derive(Deserialize, Clone, Debug)]
struct Block {
    id: u16,
    item_id: u16,
    wall_variant_id: Option<u16>,
    translation_key: String,
    name: String,
    properties: Vec<Property>,
    default_state_id: u16,
    states: Vec<State>,
}

#[derive(Deserialize, Clone, Debug)]
struct BlockEntityKind {
    id: u32,
    ident: String,
    name: String,
}

#[derive(Deserialize, Clone, Debug)]
struct Property {
    name: String,
    values: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct State {
    id: u16,
    luminance: u8,
    opaque: bool,
    replaceable: bool,
    blocks_motion: bool,
    collision_shapes: Vec<u16>,
    block_entity_type: Option<u32>,
}

#[derive(Deserialize, Clone, Debug)]
struct Shape {
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
}

static BLOCKS: LazyLock<HashMap<String, Block>> = LazyLock::new(|| {
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
    let categories: &HashSet<&str> = &BLOCKS.values().map(|val| val.name.as_str()).collect();

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

    let id = block_info.id;

    if std::env::var("CARGO_PKG_NAME").unwrap() == "pumpkin-world" {
        let category_name: proc_macro2::TokenStream = format!(
            "crate::block::BlockCategory::{}",
            &pascal_case(
                block_info
                    .name
                    .category
                    .split_once(':')
                    .expect("Bad minecraft id")
                    .1,
            )
        )
        .parse()
        .unwrap();
        let block_name: proc_macro2::TokenStream =
            format!("crate::block::Block::{}", &pascal_case(block_name.as_str()))
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
            &pascal_case(block_info.name.split_once(':').expect("Bad minecraft id").1,)
        )
        .parse()
        .unwrap();
        let block_name: proc_macro2::TokenStream = format!(
            "pumpkin_world::block::Block::{}",
            &pascal_case(block_name.split_once(':').expect("Bad minecraft id").1,)
        )
        .parse()
        .unwrap();
        quote! {
          pumpkin_world::block::BlockState {
                state_id: #id,
                category: #category_name,
                block: #block_name,
          }
        }
        .into()
    }
}
