use crate::VarInt;
use pumpkin_core::text::TextComponent;

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
