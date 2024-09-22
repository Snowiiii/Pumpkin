use proc_macro::TokenStream;
use quote::quote;

extern crate proc_macro;
#[proc_macro_attribute]
pub fn packet(input: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();

    let name = &ast.ident;

    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let input: proc_macro2::TokenStream = input.into();
    let item: proc_macro2::TokenStream = item.into();

    let gen = quote! {
        #item
        impl #impl_generics crate::bytebuf::packet_id::Packet for #name #ty_generics {
            const PACKET_ID: i32 = #input;
        }
    };

    gen.into()
}

mod block_id;
#[proc_macro]
pub fn block_id(item: TokenStream) -> TokenStream {
    block_id::block_id_impl(item)
}
