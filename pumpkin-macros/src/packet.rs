use std::{collections::HashMap, sync::LazyLock};

use proc_macro::TokenStream;
use quote::quote;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Packets {
    serverbound: HashMap<String, Vec<String>>,
    clientbound: HashMap<String, Vec<String>>,
}

static PACKETS: LazyLock<Packets> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/packets.json"))
        .expect("Could not parse packets.json registry.")
});

pub(crate) fn packet_clientbound(item: TokenStream) -> proc_macro2::TokenStream {
    let input_string = item.to_string();
    let packet_name = input_string.trim_matches('"');
    let packet_name_split: Vec<&str> = packet_name.split(":").collect();
    let phase = PACKETS
        .clientbound
        .get(packet_name_split[0])
        .expect("Invalid Phase");
    let id = phase
        .iter()
        .enumerate()
        .find(|s| s.1 == packet_name_split[1])
        .map(|(i, _)| i)
        .expect("Invalid Packet name");
    quote! { #id }
}

pub(crate) fn packet_serverbound(item: TokenStream) -> proc_macro2::TokenStream {
    let input_string = item.to_string();
    let packet_name = input_string.trim_matches('"');
    let packet_name_split: Vec<&str> = packet_name.split(":").collect();

    let phase = PACKETS
        .serverbound
        .get(packet_name_split[0])
        .expect("Invalid Phase");
    let id = phase
        .iter()
        .enumerate()
        .find(|s| s.1 == packet_name_split[1])
        .map(|(i, _)| i)
        .expect("Invalid Packet name");
    quote! { #id }
}
