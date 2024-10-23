use proc_macro::TokenStream;
use quote::quote;

extern crate proc_macro;
#[proc_macro_attribute]
pub fn client_packet(input: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();

    let name = &ast.ident;

    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let input: proc_macro2::TokenStream = input.into();
    let item: proc_macro2::TokenStream = item.into();

    let gen = quote! {
        #item
        impl #impl_generics crate::bytebuf::packet_id::ClientPacketID for #name #ty_generics {
            const PACKET_ID: i32 = #input;
        }
    };

    gen.into()
}

mod block_state;
#[proc_macro]
pub fn block(item: TokenStream) -> TokenStream {
    block_state::block_state_impl(item)
}

#[proc_macro]
/// Creates an enum for all block types. Should only be used once
pub fn blocks_enum(_item: TokenStream) -> TokenStream {
    block_state::block_enum_impl()
}

#[proc_macro]
/// Creates an enum for all block categories. Should only be used once
pub fn block_categories_enum(_item: TokenStream) -> TokenStream {
    block_state::block_type_enum_impl()
}
