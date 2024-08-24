use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};

/// Text color
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Color {
    /// The default color for the text will be used, which varies by context
    /// (in some cases, it's white; in others, it's black; in still others, it
    /// is a shade of gray that isn't normally used on text).
    #[default]
    Reset,
    /// One of the 16 named Minecraft colors
    Named(NamedColor),
}

impl Color {
    pub fn console_color(&self, text: &str) -> ColoredString {
        match self {
            Color::Reset => text.clear(),
            Color::Named(color) => match color {
                NamedColor::Black => text.black(),
                NamedColor::DarkBlue => text.blue(),
                NamedColor::DarkGreen => text.green(),
                NamedColor::DarkAqua => text.cyan(),
                NamedColor::DarkRed => text.red(),
                NamedColor::DarkPurple => text.purple(),
                NamedColor::Gold => text.yellow(),
                NamedColor::Gray => text.bright_black(),
                NamedColor::DarkGray => text.bright_black(), // ?
                NamedColor::Blue => text.bright_blue(),
                NamedColor::Green => text.bright_green(),
                NamedColor::Aqua => text.cyan(),
                NamedColor::Red => text.red(),
                NamedColor::LightPurple => text.bright_purple(),
                NamedColor::Yellow => text.bright_yellow(),
                NamedColor::White => text.white(),
            },
        }
    }
}

/// Named Minecraft color
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedColor {
    Black = 0,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}
