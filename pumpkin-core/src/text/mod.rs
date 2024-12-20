use core::str;
use std::borrow::Cow;

use crate::text::color::ARGBColor;
use click::ClickEvent;
use color::Color;
use colored::Colorize;
use hover::HoverEvent;
use serde::{Deserialize, Serialize};
use style::Style;

pub mod click;
pub mod color;
pub mod hover;
pub mod style;

/// Represents a Text component
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Extra text components
    pub extra: Vec<TextComponent<'a>>,
}

impl<'a> TextComponent<'a> {
    pub fn text(text: &'a str) -> Self {
        Self {
            content: TextContent::Text { text: text.into() },
            style: Style::default(),
            extra: vec![],
        }
    }

    pub fn text_string(text: String) -> Self {
        Self {
            content: TextContent::Text { text: text.into() },
            style: Style::default(),
            extra: vec![],
        }
    }

    pub fn add_child(mut self, child: TextComponent<'a>) -> Self {
        self.extra.push(child);
        self
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
        if style.click_event.is_some() {
            if let Some(ClickEvent::OpenUrl(url)) = style.click_event {
                //TODO: check if term supports hyperlinks before
                text = format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text).to_string()
            }
        }
        for child in self.extra {
            text += &*child.to_pretty_console();
        }
        text
    }
}

impl serde::Serialize for TextComponent<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.encode())
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

    pub fn color_rgb(mut self, color: color::RGBColor) -> Self {
        self.style.color = Some(Color::Rgb(color));
        self
    }

    /// Makes the text bold
    pub fn bold(mut self) -> Self {
        self.style.bold = Some(true);
        self
    }

    /// Makes the text italic
    pub fn italic(mut self) -> Self {
        self.style.italic = Some(true);
        self
    }

    /// Makes the text underlined
    pub fn underlined(mut self) -> Self {
        self.style.underlined = Some(true);
        self
    }

    /// Makes the text strikethrough
    pub fn strikethrough(mut self) -> Self {
        self.style.strikethrough = Some(true);
        self
    }

    /// Makes the text obfuscated
    pub fn obfuscated(mut self) -> Self {
        self.style.obfuscated = Some(true);
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

    /// Allows you to change the font of the text.
    /// Default fonts: `minecraft:default`, `minecraft:uniform`, `minecraft:alt`, `minecraft:illageralt`
    pub fn font(mut self, identifier: String) -> Self {
        self.style.font = Some(identifier);
        self
    }

    /// Overrides the shadow properties of text.
    pub fn shadow_color(mut self, color: ARGBColor) -> Self {
        self.style.shadow_color = Some(color);
        self
    }

    pub fn encode(&self) -> bytes::BytesMut {
        // TODO: Somehow fix this ugly mess
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct TempStruct<'a> {
            #[serde(flatten)]
            text: &'a TextContent<'a>,
            #[serde(flatten)]
            style: &'a Style<'a>,
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            #[serde(rename = "extra")]
            extra: Vec<TempStruct<'a>>,
        }
        fn convert_extra<'a>(extra: &'a [TextComponent<'a>]) -> Vec<TempStruct<'a>> {
            extra
                .iter()
                .map(|x| TempStruct {
                    text: &x.content,
                    style: &x.style,
                    extra: convert_extra(&x.extra),
                })
                .collect()
        }

        let temp_extra = convert_extra(&self.extra);
        let astruct = TempStruct {
            text: &self.content,
            style: &self.style,
            extra: temp_extra,
        };
        // dbg!(&serde_json::to_string(&astruct));
        // dbg!(pumpkin_nbt::serializer::to_bytes_unnamed(&astruct).unwrap().to_vec());

        // TODO
        pumpkin_nbt::serializer::to_bytes_unnamed(&astruct).unwrap()
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
        with: Vec<TextComponent<'a>>,
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
