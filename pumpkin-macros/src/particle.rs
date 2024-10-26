use std::{collections::HashMap, sync::LazyLock};

use proc_macro::TokenStream;
use quote::quote;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Particle {
    name: String,
    id: u32,
}

static SOUNDS: LazyLock<HashMap<String, u32>> = LazyLock::new(|| {
    serde_json::from_str::<Vec<Particle>>(include_str!("../../assets/particles.json"))
        .expect("Could not parse particles.json registry.")
        .into_iter()
        .map(|val| (val.name, val.id))
        .collect()
});

pub(crate) fn particle_impl(item: TokenStream) -> TokenStream {
    let input_string = item.to_string();
    let sound_name = input_string.trim_matches('"');

    let id = SOUNDS.get(sound_name).expect("Invalid particle");
    quote! { #id }.into()
}
