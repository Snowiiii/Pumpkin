use std::{collections::HashMap, sync::LazyLock};

use proc_macro::TokenStream;
use quote::quote;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Sound {
    name: String,
    id: u32,
}

static SOUNDS: LazyLock<HashMap<String, u32>> = LazyLock::new(|| {
    serde_json::from_str::<Vec<Sound>>(include_str!("../../assets/sounds.json"))
        .expect("Could not parse sounds.json registry.")
        .into_iter()
        .map(|val| (val.name, val.id))
        .collect()
});

pub(crate) fn sound_impl(item: TokenStream) -> TokenStream {
    let input_string = item.to_string();
    let sound_name = input_string.trim_matches('"');

    let id = SOUNDS.get(sound_name).expect("Invalid sound");
    quote! { #id }.into()
}
