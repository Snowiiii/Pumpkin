use std::{collections::HashMap, sync::LazyLock};

use proc_macro::TokenStream;
use quote::quote;

static PARTICLES: LazyLock<HashMap<String, u16>> = LazyLock::new(|| {
    serde_json::from_str::<Vec<String>>(include_str!("../../assets/particles.json"))
        .expect("Could not parse particles.json registry")
        .into_iter()
        .enumerate()
        .map(|(i, s)| (s, i as u16))
        .collect()
});

pub(crate) fn particle_impl(item: TokenStream) -> TokenStream {
    let input_string = item.to_string();
    let particle_name = input_string.trim_matches('"');

    let id = *PARTICLES.get(particle_name).expect("Invalid Particle");
    quote! { #id }.into()
}
