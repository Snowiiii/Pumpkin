use std::{collections::HashMap, sync::LazyLock};

use proc_macro::TokenStream;
use quote::quote;

static SCREENS: LazyLock<HashMap<String, u8>> = LazyLock::new(|| {
    serde_json::from_str::<Vec<String>>(include_str!("../../assets/screens.json"))
        .expect("Could not parse screens.json registry.")
        .into_iter()
        .enumerate()
        .map(|(i, s)| (s, i as u8))
        .collect()
});

pub(crate) fn screen_impl(item: TokenStream) -> TokenStream {
    let input_string = item.to_string();
    let screen_name = input_string.trim_matches('"');

    let id = *SCREENS.get(screen_name).expect("Invalid Screen");
    quote! { #id }.into()
}
