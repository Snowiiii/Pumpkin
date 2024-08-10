use core::str;

use click::ClickEvent;
use color::Color;
use fastnbt::SerOpts;
use hover::HoverEvent;
use serde::{Deserialize, Serialize};

pub mod click;
pub mod color;
pub mod hover;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Text(pub Box<TextComponent>);

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
    /// Allows for events to occur when the player clicks on text. Only work in chat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub click_event: Option<ClickEvent>,
    /// Allows for a tooltip to be displayed when the player hovers their mouse over text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hover_event: Option<HoverEvent>,
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

    pub fn color_named(mut self, color: color::NamedColor) -> Self {
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

    /// Allows for events to occur when the player clicks on text. Only work in chat.
    pub fn click_event(mut self, event: ClickEvent) -> Self {
        self.click_event = Some(event);
        self
    }

    /// Allows for a tooltip to be displayed when the player hovers their mouse over text.
    pub fn hover_event(mut self, event: HoverEvent) -> Self {
        self.hover_event = Some(event);
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
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub click_event: &'a Option<ClickEvent>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub hover_event: &'a Option<HoverEvent>,
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
            click_event: &self.click_event,
            hover_event: &self.hover_event,
        };
        // dbg!(&serde_json::to_string(&astruct));

        fastnbt::to_bytes_with_opts(&astruct, SerOpts::network_nbt()).unwrap()
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
            click_event: None,
            hover_event: None,
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
            click_event: None,
            hover_event: None,
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
