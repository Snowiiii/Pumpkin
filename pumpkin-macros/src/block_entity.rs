use proc_macro::TokenStream;
use quote::quote;
use serde::Deserialize;
use std::{collections::HashMap, sync::LazyLock};

#[derive(Deserialize)]
pub struct TopLevel {
    block_entity_types: Vec<BlockEntityKind>,
}

#[expect(dead_code)]
#[derive(Deserialize)]
struct BlockEntityKind {
    id: u32,
    ident: String,
    name: String,
}
static BLOCK_ENTITIES: LazyLock<HashMap<String, u32>> = LazyLock::new(|| {
    serde_json::from_str::<TopLevel>(include_str!("../../assets/blocks.json"))
        .expect("Could not parse blocks.json registry.")
        .block_entity_types
        .into_iter()
        .map(|value| (value.ident, value.id))
        .collect()
});

pub(crate) fn block_entity_impl(item: TokenStream) -> TokenStream {
    let input_string = item.to_string();
    let block_entity_name = input_string.trim_matches('"');

    let id = BLOCK_ENTITIES
        .get(block_entity_name)
        .expect("Invalid block");
    quote! { #id }.into()
}
