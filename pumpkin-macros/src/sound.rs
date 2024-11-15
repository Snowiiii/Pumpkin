use proc_macro::TokenStream;
use pumpkin_core::sound::SOUNDS;
use quote::quote;

pub(crate) fn sound_impl(item: TokenStream) -> TokenStream {
    let input_string = item.to_string();
    let sound_name = input_string.trim_matches('"');

    let id = SOUNDS.get(sound_name).expect("Invalid sound");
    quote! { #id }.into()
}
