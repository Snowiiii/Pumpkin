use proc_macro::TokenStream;
use quote::quote;
use serde_json::Value;
use std::{collections::HashMap, sync::LazyLock};

static GENERIC_BLOCKS: LazyLock<HashMap<String, u32>> = LazyLock::new(|| {
    serde_json::from_str::<Value>(include_str!("../../assets/blocks.json"))
        .expect("")
        .get("block_entity_types")
        .and_then(|arr| arr.as_array())
        .expect("")
        .iter()
        .map(|value| {
            (
                value["ident"].as_str().expect("").to_string(),
                value["id"].as_u64().expect("") as u32,
            )
        })
        .collect()
});

pub(crate) fn block_impl(item: TokenStream) -> TokenStream {
    let input_string = item.to_string();
    let block_entity_name = input_string.trim_matches('"');

    let id = GENERIC_BLOCKS
        .get(block_entity_name)
        .expect("Invalid block");
    quote! { #id }.into()
}
