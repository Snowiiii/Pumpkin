use serde::{Deserialize, Serialize};

use super::{
    click::ClickEvent,
    color::{self, Color},
    hover::HoverEvent,
};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct Style<'a> {
    /// Changes the color to render the content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bold: Option<u8>,
    /// Whether to render the content in italic.
    /// Keep in mind that booleans are represented as bytes in nbt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub italic: Option<u8>,
    /// Whether to render the content in underlined.
    /// Keep in mind that booleans are represented as bytes in nbt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub underlined: Option<u8>,
    /// Whether to render the content in strikethrough.
    /// Keep in mind that booleans are represented as bytes in nbt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<u8>,
    /// Whether to render the content in obfuscated.
    /// Keep in mind that booleans are represented as bytes in nbt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obfuscated: Option<u8>,
    /// When the text is shift-clicked by a player, this string is inserted in their chat input. It does not overwrite any existing text the player was writing. This only works in chat messages
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insertion: Option<String>,
    /// Allows for events to occur when the player clicks on text. Only work in chat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub click_event: Option<ClickEvent<'a>>,
    /// Allows for a tooltip to be displayed when the player hovers their mouse over text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hover_event: Option<HoverEvent<'a>>,
}

impl<'a> Style<'a> {
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
    pub fn click_event(mut self, event: ClickEvent<'a>) -> Self {
        self.click_event = Some(event);
        self
    }

    /// Allows for a tooltip to be displayed when the player hovers their mouse over text.
    pub fn hover_event(mut self, event: HoverEvent<'a>) -> Self {
        self.hover_event = Some(event);
        self
    }
}
