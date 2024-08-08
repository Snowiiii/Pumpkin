use core::str;

use fastnbt::SerOpts;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Text(Box<TextComponent>);

// Fepresents a Text component
// Reference: https://wiki.vg/Text_formatting#Text_components
#[derive(Clone, Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextComponent {
    /// The actual text
    #[serde(flatten)]
    pub content: TextContent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    /// Changes the color to render the content
    pub color: Option<Color>,
    /// Whether to render the content in bold.
    /// Keep in mind that booleans are representet as bytes in nbt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bold: Option<u8>,
    /// Whether to render the content in italic.
    /// Keep in mind that booleans are representet as bytes in nbt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub italic: Option<u8>,
    /// Whether to render the content in underlined.
    /// Keep in mind that booleans are representet as bytes in nbt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub underlined: Option<u8>,
    /// Whether to render the content in strikethrough.
    /// Keep in mind that booleans are representet as bytes in nbt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<u8>,
    /// Whether to render the content in obfuscated.
    /// Keep in mind that booleans are representet as bytes in nbt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obfuscated: Option<u8>,
    /// When the text is shift-clicked by a player, this string is inserted in their chat input. It does not overwrite any existing text the player was writing. This only works in chat messages
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insertion: Option<String>,
}

impl serde::Serialize for TextComponent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.encode().as_slice())
    }
}

impl TextComponent {
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn color_named(mut self, color: NamedColor) -> Self {
        self.color = Some(Color::Named(color));
        self
    }

    /// Makes the text bold
    pub fn bold(mut self) -> Self {
        self.bold = Some(1);
        self
    }

    /// Makes the text italic
    pub fn italic(mut self) -> Self {
        self.italic = Some(1);
        self
    }

    /// Makes the text underlined
    pub fn underlined(mut self) -> Self {
        self.underlined = Some(1);
        self
    }

    /// Makes the text strikethrough
    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = Some(1);
        self
    }

    /// Makes the text obfuscated
    pub fn obfuscated(mut self) -> Self {
        self.obfuscated = Some(1);
        self
    }

    /// When the text is shift-clicked by a player, this string is inserted in their chat input. It does not overwrite any existing text the player was writing. This only works in chat messages
    pub fn insertion(mut self, text: String) -> Self {
        self.insertion = Some(text);
        self
    }

    pub fn encode(&self) -> Vec<u8> {
        // TODO: Somehow fix this ugly mess
        #[derive(serde::Serialize, Debug)]
        #[serde(rename_all = "camelCase")]
        struct TempStruct<'a> {
            #[serde(flatten)]
            text: &'a TextContent,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub color: &'a Option<Color>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub bold: &'a Option<u8>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub italic: &'a Option<u8>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub underlined: &'a Option<u8>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub strikethrough: &'a Option<u8>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub obfuscated: &'a Option<u8>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub insertion: &'a Option<String>,
        }
        let astruct = TempStruct {
            text: &self.content,
            color: &self.color,
            bold: &self.bold,
            italic: &self.italic,
            underlined: &self.underlined,
            strikethrough: &self.strikethrough,
            obfuscated: &self.obfuscated,
            insertion: &self.insertion,
        };
        let nbt = fastnbt::to_bytes_with_opts(&astruct, SerOpts::network_nbt()).unwrap();
        nbt
    }
}

impl From<String> for TextComponent {
    fn from(value: String) -> Self {
        Self {
            content: TextContent::Text { text: value },
            color: None,
            bold: None,
            italic: None,
            underlined: None,
            strikethrough: None,
            obfuscated: None,
            insertion: None,
        }
    }
}

impl From<&str> for TextComponent {
    fn from(value: &str) -> Self {
        Self {
            content: TextContent::Text {
                text: value.to_string(),
            },
            color: None,
            bold: None,
            italic: None,
            underlined: None,
            strikethrough: None,
            obfuscated: None,
            insertion: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextContent {
    /// Raw Text
    Text { text: String },
    /// Translated text
    Translate {
        translate: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        with: Vec<Text>,
    },
    /// Displays the name of one or more entities found by a selector.
    EntityNames {
        selector: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        separator: Option<Text>,
    },
    /// A keybind identifier
    /// https://minecraft.fandom.com/wiki/Controls#Configurable_controls
    Keybind { keybind: String },
}

impl Default for TextContent {
    fn default() -> Self {
        Self::Text { text: "".into() }
    }
}

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
