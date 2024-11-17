use pumpkin_core::text::TextComponent;
use crate::VarInt;

pub enum BosseventAction<'a> {
    Add {
        title: TextComponent<'a>,
        health: f32,
        color: VarInt,
        division: VarInt,
        flags: u8,
    },
    Remove,
    UpdateHealth(f32),
    UpdateTile(TextComponent<'a>),
    UpdateStyle {
        color: VarInt,
        dividers: VarInt,
    },
    UpdateFlags(u8),
}