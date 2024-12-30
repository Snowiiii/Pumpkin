use std::{collections::HashMap, sync::LazyLock};

use proc_macro::TokenStream;
use quote::quote;

static SOUNDS: LazyLock<HashMap<String, u16>> = LazyLock::new(|| {
    serde_json::from_str::<Vec<String>>(include_str!("../../assets/sounds.json"))
        .expect("Could not parse sounds.json registry.")
        .into_iter()
        .enumerate()
        .map(|(i, s)| (s, i as u16))
        .collect()
});

pub(crate) fn sound_impl(item: TokenStream) -> TokenStream {
    let input_string = item.to_string();
    let sound_name = input_string.trim_matches('"');

    let id = *SOUNDS.get(sound_name).expect("Invalid sound");
    quote! { #id }.into()
}
