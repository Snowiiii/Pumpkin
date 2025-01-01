use crate::VarInt;
use pumpkin_core::text::TextComponent;

pub enum BosseventAction {
    Add {
        title: TextComponent,
        health: f32,
        color: VarInt,
        division: VarInt,
        flags: u8,
    },
    Remove,
    UpdateHealth(f32),
    UpdateTile(TextComponent),
    UpdateStyle {
        color: VarInt,
        dividers: VarInt,
    },
    UpdateFlags(u8),
}
