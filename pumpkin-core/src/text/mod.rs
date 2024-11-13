use core::str;
use std::borrow::Cow;

use click::ClickEvent;
use color::Color;
use colored::Colorize;
use fastnbt::SerOpts;
use hover::HoverEvent;
use serde::{Deserialize, Serialize};
use style::Style;

pub mod click;
pub mod color;
pub mod hover;
pub mod style;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Text<'a>(pub Box<TextComponent<'a>>);

// Represents a Text component
// Reference: https://wiki.vg/Text_formatting#Text_components
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct TextComponent<'a> {
    /// The actual text
    #[serde(flatten)]
    pub content: TextContent<'a>,
    /// Style of the text. Bold, Italic, underline, Color...
    /// Also has `ClickEvent
    #[serde(flatten)]
    pub style: Style<'a>,
}

impl<'a> TextComponent<'a> {
    pub fn text(text: &'a str) -> Self {
        Self {
            content: TextContent::Text { text: text.into() },
            style: Style::default(),
        }
    }

    pub fn text_string(text: String) -> Self {
        Self {
            content: TextContent::Text { text: text.into() },
            style: Style::default(),
        }
    }

    pub fn to_pretty_console(self) -> String {
        let style = self.style;
        let color = style.color;
        let mut text = match self.content {
            TextContent::Text { text } => text.into_owned(),
            TextContent::Translate { translate, with: _ } => translate.into_owned(),
            TextContent::EntityNames {
                selector,
                separator: _,
            } => selector.into_owned(),
            TextContent::Keybind { keybind } => keybind.into_owned(),
        };
        if let Some(color) = color {
            text = color.console_color(&text).to_string();
        }
        if style.bold.is_some() {
            text = text.bold().to_string();
        }
        if style.italic.is_some() {
            text = text.italic().to_string();
        }
        if style.underlined.is_some() {
            text = text.underline().to_string();
        }
        if style.strikethrough.is_some() {
            text = text.strikethrough().to_string();
        }
        text
    }
}

impl<'a> serde::Serialize for TextComponent<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(self.encode().as_slice())
    }
}

impl<'a> TextComponent<'a> {
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
    pub fn click_event(mut self, event: ClickEvent<'a>) -> Self {
        self.style.click_event = Some(event);
        self
    }

    /// Allows for a tooltip to be displayed when the player hovers their mouse over text.
    pub fn hover_event(mut self, event: HoverEvent<'a>) -> Self {
        self.style.hover_event = Some(event);
        self
    }

    pub fn encode(&self) -> Vec<u8> {
        // TODO: Somehow fix this ugly mess
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct TempStruct<'a> {
            #[serde(flatten)]
            text: &'a TextContent<'a>,
            #[serde(flatten)]
            style: &'a Style<'a>,
        }
        let astruct = TempStruct {
            text: &self.content,
            style: &self.style,
        };
        // dbg!(&serde_json::to_string(&astruct));

        fastnbt::to_bytes_with_opts(&astruct, SerOpts::network_nbt()).unwrap()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum TextContent<'a> {
    /// Raw Text
    Text { text: Cow<'a, str> },
    /// Translated text
    Translate {
        translate: Cow<'a, str>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        with: Vec<Text<'a>>,
    },
    /// Displays the name of one or more entities found by a selector.
    EntityNames {
        selector: Cow<'a, str>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        separator: Option<Cow<'a, str>>,
    },
    /// A keybind identifier
    /// https://minecraft.wiki/w/Controls#Configurable_controls
    Keybind { keybind: Cow<'a, str> },
}
