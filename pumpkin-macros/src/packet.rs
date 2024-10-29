use std::{collections::HashMap, sync::LazyLock};

use proc_macro::TokenStream;
use quote::quote;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Packet {
    name: String,
    phase: String,
    side: String,
    id: u16,
}

static PACKETS: LazyLock<HashMap<String, u16>> = LazyLock::new(|| {
    serde_json::from_str::<Vec<Packet>>(include_str!("../../assets/packets.json"))
        .expect("Could not parse packets.json registry.")
        .into_iter()
        .map(|val| (format!("{}:{}:{}", val.side, val.phase, val.name), val.id))
        .collect()
});

pub(crate) fn packet_clientbound(item: TokenStream) -> proc_macro2::TokenStream {
    let input_string = item.to_string();
    let packet_name = input_string.trim_matches('"');

    let id = PACKETS
        .get(&format!("clientbound:{}", packet_name))
        .expect("Invalid Packet");
    quote! { #id }
}

#[expect(dead_code)]
pub(crate) fn packet_serverbound(item: TokenStream) -> proc_macro2::TokenStream {
    let input_string = item.to_string();
    let packet_name = input_string.trim_matches('"');

    let id = PACKETS
        .get(&format!("serverbound:{}", packet_name))
        .expect("Invalid Packet");
    quote! { #id }
}
