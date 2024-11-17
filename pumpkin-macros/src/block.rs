use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn block_entity_impl(item: TokenStream) -> TokenStream {
    let input_string = item.to_string();
    let block_entity_name = input_string.trim_matches('"');

    quote! {
        pumpkin_world::block::block_registry::BLOCKS
            .block_entity_types
            .iter()
            .find(|block_type| block_type.ident == #block_entity_name)
            .unwrap()
            .id
    }
    .into()
}
