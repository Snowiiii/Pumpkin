use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::{NumberFormat, VarInt};

use super::ClientboundPlayPackets;

#[derive(Serialize)]
#[client_packet(ClientboundPlayPackets::UpdateScore as i32)]
pub struct CUpdateScore<'a> {
    entity_name: &'a str,
    objective_name: &'a str,
    value: VarInt,
    display_name: Option<TextComponent<'a>>,
    number_format: Option<NumberFormat<'a>>,
}

impl<'a> CUpdateScore<'a> {
    pub fn new(
        entity_name: &'a str,
        objective_name: &'a str,
        value: VarInt,
        display_name: Option<TextComponent<'a>>,
        number_format: Option<NumberFormat<'a>>,
    ) -> Self {
        Self {
            entity_name,
            objective_name,
            value,
            display_name,
            number_format,
        }
    }
}
