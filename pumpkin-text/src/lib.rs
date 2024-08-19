use core::str;

use click::ClickEvent;
use color::Color;
use fastnbt::SerOpts;
use hover::HoverEvent;
use serde::{Deserialize, Serialize};
use style::Style;

pub mod click;
pub mod color;
pub mod hover;
pub mod style;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Text(pub Box<TextComponent>);

// Represents a Text component
// Reference: https://wiki.vg/Text_formatting#Text_components
#[derive(Clone, Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextComponent {
    /// The actual text
    #[serde(flatten)]
    pub content: TextContent,
    /// Style of the text. Bold, Italic, underline, Color...
    /// Also has `ClickEvent
    #[serde(flatten)]
    pub style: Style,
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
        self.style.color = Some(color);
        self
    }

    pub fn color_named(mut self, color: color::NamedColor) -> Self {
        self.style.color = Some(Color::Named(color));
        self
    }

    /// Makes the text bold
    pub fn bold(mut self) -> Self {
        self.style.bold = Some(1);
        self
    }

    /// Makes the text italic
    pub fn italic(mut self) -> Self {
        self.style.italic = Some(1);
        self
    }

    /// Makes the text underlined
    pub fn underlined(mut self) -> Self {
        self.style.underlined = Some(1);
        self
    }

    /// Makes the text strikethrough
    pub fn strikethrough(mut self) -> Self {
        self.style.strikethrough = Some(1);
        self
    }

    /// Makes the text obfuscated
    pub fn obfuscated(mut self) -> Self {
        self.style.obfuscated = Some(1);
        self
    }

    /// When the text is shift-clicked by a player, this string is inserted in their chat input. It does not overwrite any existing text the player was writing. This only works in chat messages
    pub fn insertion(mut self, text: String) -> Self {
        self.style.insertion = Some(text);
        self
    }

    /// Allows for events to occur when the player clicks on text. Only work in chat.
    pub fn click_event(mut self, event: ClickEvent) -> Self {
        self.style.click_event = Some(event);
        self
    }

    /// Allows for a tooltip to be displayed when the player hovers their mouse over text.
    pub fn hover_event(mut self, event: HoverEvent) -> Self {
        self.style.hover_event = Some(event);
        self
    }

    pub fn encode(&self) -> Vec<u8> {
        // TODO: Somehow fix this ugly mess
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct TempStruct<'a> {
            #[serde(flatten)]
            text: &'a TextContent,
            #[serde(flatten)]
            style: &'a Style,
        }
        let astruct = TempStruct {
            text: &self.content,
            style: &self.style,
        };
        // dbg!(&serde_json::to_string(&astruct));

        fastnbt::to_bytes_with_opts(&astruct, SerOpts::network_nbt()).unwrap()
    }
}

impl From<String> for TextComponent {
    fn from(value: String) -> Self {
        Self {
            content: TextContent::Text { text: value },
            style: Style::default(),
        }
    }
}

impl From<&str> for TextComponent {
    fn from(value: &str) -> Self {
        Self {
            content: TextContent::Text {
                text: value.to_string(),
            },
            style: Style::default(),
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
