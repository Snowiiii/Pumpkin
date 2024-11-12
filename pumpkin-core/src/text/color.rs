use colored::{ColoredString, Colorize};
use serde::{Deserialize, Deserializer, Serialize};

/// Text color
#[derive(Default, Debug, Clone, Copy, Serialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Color {
    /// The default color for the text will be used, which varies by context
    /// (in some cases, it's white; in others, it's black; in still others, it
    /// is a shade of gray that isn't normally used on text).
    #[default]
    Reset,
    /// RGB Color
    Rgb(u32),
    /// One of the 16 named Minecraft colors
    Named(NamedColor),
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if s == "reset" {
            Ok(Color::Reset)
        } else if s.starts_with('#') {
            let rgb = u32::from_str_radix(&s.replace("#", ""), 16)
                .map_err(|_| serde::de::Error::custom("Invalid hex color"))?;
            Ok(Color::Rgb(rgb))
        } else {
            Ok(Color::Named(NamedColor::try_from(s.as_str()).map_err(
                |_| serde::de::Error::custom("Invalid named color"),
            )?))
        }
    }
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
            Color::Rgb(_) => todo!(),
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

impl TryFrom<&str> for NamedColor {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "black" => Ok(NamedColor::Black),
            "dark_blue" => Ok(NamedColor::DarkBlue),
            "dark_green" => Ok(NamedColor::DarkGreen),
            "dark_aqua" => Ok(NamedColor::DarkAqua),
            "dark_red" => Ok(NamedColor::DarkRed),
            "dark_purple" => Ok(NamedColor::DarkPurple),
            "gold" => Ok(NamedColor::Gold),
            "gray" => Ok(NamedColor::Gray),
            "dark_gray" => Ok(NamedColor::DarkGray),
            "blue" => Ok(NamedColor::Blue),
            "green" => Ok(NamedColor::Green),
            "aqua" => Ok(NamedColor::Aqua),
            "red" => Ok(NamedColor::Red),
            "light_purple" => Ok(NamedColor::LightPurple),
            "yellow" => Ok(NamedColor::Yellow),
            "white" => Ok(NamedColor::White),
            _ => Err(()),
        }
    }
}
