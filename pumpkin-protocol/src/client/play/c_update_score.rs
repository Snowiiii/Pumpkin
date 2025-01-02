use pumpkin_core::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::{NumberFormat, VarInt};

#[derive(Serialize)]
#[client_packet("play:set_score")]
pub struct CUpdateScore<'a> {
    entity_name: &'a str,
    objective_name: &'a str,
    value: VarInt,
    display_name: Option<TextComponent>,
    number_format: Option<NumberFormat>,
}

impl<'a> CUpdateScore<'a> {
    pub fn new(
        entity_name: &'a str,
        objective_name: &'a str,
        value: VarInt,
        display_name: Option<TextComponent>,
        number_format: Option<NumberFormat>,
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
