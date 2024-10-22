use std::{collections::HashMap, sync::LazyLock};

use proc_macro::TokenStream;
use quote::quote;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Particle {
    name: String,
    id: i32,
}

static SOUNDS: LazyLock<HashMap<String, i32>> = LazyLock::new(|| {
    serde_json::from_str::<Vec<Particle>>(include_str!("../../assets/particles.json"))
        .expect("Could not parse sounds.json registry.")
        .into_iter()
        .map(|val| (val.name, val.id))
        .collect()
});

pub(crate) fn particle_impl(item: TokenStream) -> TokenStream {
    let input_string = item.to_string();
    let sound_name = input_string.trim_matches('"');

    let id = SOUNDS.get(sound_name).expect("Invalid sound");
    quote! { #id }.into()
}
