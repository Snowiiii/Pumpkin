use core::str;

use fastnbt::SerOpts;
use serde::{Deserialize, Serialize};

// represents a text component
// Reference: https://wiki.vg/Text_formatting#Text_components
#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct TextComponent {
    pub text: String,
}

impl TextComponent {
    pub fn encode(&self) -> Vec<u8> {
        fastnbt::to_bytes_with_opts(&self, SerOpts::network_nbt()).unwrap()
    }
}

impl From<String> for TextComponent {
    fn from(value: String) -> Self {
        Self { text: value }
    }
}

impl From<&str> for TextComponent {
    fn from(value: &str) -> Self {
        Self {
            text: value.to_string(),
        }
    }
}

/// Text color
#[derive(Default, Debug, Clone, Copy)]
pub enum Color {
    /// The default color for the text will be used, which varies by context
    /// (in some cases, it's white; in others, it's black; in still others, it
    /// is a shade of gray that isn't normally used on text).
    #[default]
    Reset,
    /// One of the 16 named Minecraft colors
    Named(NamedColor),
}

/// Named Minecraft color
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum NamedColor {
    Black = 0,
    DarkBlue,
    DarkGreen,
    DarkCyan,
    DarkRed,
    Purple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    BrightGreen,
    Cyan,
    Red,
    Pink,
    Yellow,
    White,
}
