use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Action to take on click of the text.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "action", content = "value", rename_all = "snake_case")]
pub enum ClickEvent<'a> {
    /// Opens a URL
    OpenUrl(Cow<'a, str>),
    /// Works in signs, but only on the root text component
    RunCommand(Cow<'a, str>),
    /// Replaces the contents of the chat box with the text, not necessarily a
    /// command.
    SuggestCommand(Cow<'a, str>),
    /// Only usable within written books. Changes the page of the book. Indexing
    /// starts at 1.
    ChangePage(i32),
    /// Copies the given text to system clipboard
    CopyToClipboard(Cow<'a, str>),
}
