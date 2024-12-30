use super::{
    click::ClickEvent,
    color::{self, Color},
    hover::HoverEvent,
};
use crate::text::color::ARGBColor;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Style {
    /// Changes the color to render the content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bold: Option<bool>,
    /// Whether to render the content in italic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,
    /// Whether to render the content in underlined.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub underlined: Option<bool>,
    /// Whether to render the content in strikethrough.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strikethrough: Option<bool>,
    /// Whether to render the content in obfuscated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub obfuscated: Option<bool>,
    /// When the text is shift-clicked by a player, this string is inserted in their chat input. It does not overwrite any existing text the player was writing. This only works in chat messages
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insertion: Option<String>,
    /// Allows for events to occur when the player clicks on text. Only work in chat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub click_event: Option<ClickEvent>,
    /// Allows for a tooltip to be displayed when the player hovers their mouse over text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hover_event: Option<HoverEvent>,
    /// Allows you to change the font of the text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "shadow_color"
    )]
    pub shadow_color: Option<ARGBColor>,
}

impl Style {
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
        self.bold = Some(true);
        self
    }

    /// Makes the text italic
    pub fn italic(mut self) -> Self {
        self.italic = Some(true);
        self
    }

    /// Makes the text underlined
    pub fn underlined(mut self) -> Self {
        self.underlined = Some(true);
        self
    }

    /// Makes the text strikethrough
    pub fn strikethrough(mut self) -> Self {
        self.strikethrough = Some(true);
        self
    }

    /// Makes the text obfuscated
    pub fn obfuscated(mut self) -> Self {
        self.obfuscated = Some(true);
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

    /// Allows you to change the font of the text.
    /// Default fonts: `minecraft:default`, `minecraft:uniform`, `minecraft:alt`, `minecraft:illageralt`
    pub fn font(mut self, identifier: String) -> Self {
        self.font = Some(identifier);
        self
    }

    /// Overrides the shadow properties of text.
    pub fn shadow_color(mut self, color: ARGBColor) -> Self {
        self.shadow_color = Some(color);
        self
    }
}
