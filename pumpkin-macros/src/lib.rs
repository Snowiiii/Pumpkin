use std::{io::Cursor, sync::LazyLock};

use base64::{engine::general_purpose, Engine as _};
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

static ICON: LazyLock<&[u8]> = LazyLock::new(|| include_bytes!("../../assets/default_icon.png"));
#[proc_macro]
/// Returns a base64 string encoding of the default server favicon
pub fn create_icon(_item: TokenStream) -> TokenStream {
    let icon = png::Decoder::new(Cursor::new(ICON.as_ref()));
    let reader = icon.read_info().unwrap();
    let info = reader.info();
    assert!(info.width == 64, "Icon width must be 64");
    assert!(info.height == 64, "Icon height must be 64");

    // Once we validate the dimensions, we can encode the image as-is
    let mut result = "data:image/png;base64,".to_owned();
    general_purpose::STANDARD.encode_string(ICON.as_ref(), &mut result);

    quote! {#result}.into()
}
