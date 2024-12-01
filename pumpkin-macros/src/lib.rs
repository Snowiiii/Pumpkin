use proc_macro::TokenStream;
use quote::quote;

extern crate proc_macro;

mod packet;
#[proc_macro_attribute]
pub fn client_packet(input: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let input: proc_macro2::TokenStream = packet::packet_clientbound(input);
    let item: proc_macro2::TokenStream = item.into();

    let gen = quote! {
        #item
        impl #impl_generics crate::bytebuf::packet_id::Packet for #name #ty_generics {
            const PACKET_ID: i32 = #input as i32;
        }
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn server_packet(input: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let input: proc_macro2::TokenStream = packet::packet_serverbound(input);
    let item: proc_macro2::TokenStream = item.into();

    let gen = quote! {
        #item
        impl #impl_generics crate::bytebuf::packet_id::Packet for #name #ty_generics {
            const PACKET_ID: i32 = #input as i32;
        }
    };

    gen.into()
}

mod screen;
#[proc_macro]
pub fn screen(item: TokenStream) -> TokenStream {
    screen::screen_impl(item)
}
mod sound;
#[proc_macro]
pub fn sound(item: TokenStream) -> TokenStream {
    sound::sound_impl(item)
}

mod particle;
#[proc_macro]
pub fn particle(item: TokenStream) -> TokenStream {
    particle::particle_impl(item)
}

/// clippy only gets a hash of the input so pumpkin-data doesn't have to be compiled just for clippy -- this should possibly be reconsidered
#[cfg(not(clippy))]
mod block;

#[cfg(clippy)]
#[proc_macro]
pub fn block_id(item: TokenStream) -> TokenStream {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    item.to_string().hash(&mut hasher);
    let id = hasher.finish() as u16;
    quote::quote! { #id }.into()
}

#[cfg(clippy)]
#[proc_macro]
pub fn block_state_id(item: TokenStream) -> TokenStream {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    item.to_string().hash(&mut hasher);
    let id = hasher.finish() as u16;
    quote::quote! { #id }.into()
}

#[cfg(clippy)]
#[proc_macro]
pub fn block_entity_id(item: TokenStream) -> TokenStream {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    item.to_string().hash(&mut hasher);
    let id = hasher.finish() as u32;
    quote::quote! { #id }.into()
}

#[cfg(not(clippy))]
#[proc_macro]
pub fn block_id(item: TokenStream) -> TokenStream {
    block::block_id_impl(item)
}

#[cfg(not(clippy))]
#[proc_macro]
pub fn block_state_id(item: TokenStream) -> TokenStream {
    block::block_state_id_impl(item)
}

#[cfg(not(clippy))]
#[proc_macro]
pub fn block_entity_id(item: TokenStream) -> TokenStream {
    block::block_entity_id_impl(item)
}
