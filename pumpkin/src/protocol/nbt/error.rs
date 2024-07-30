use super::Tag;
use serde::{de, ser};
use std::{fmt, fmt::Display, num::TryFromIntError};

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Message(String),
    Eof,

    TryFromInt(TryFromIntError),
    ListType(Tag, Tag),
    MapKey(Tag),
    CannotSerializeNone,
    Enum,
}

pub type Result<T> = std::result::Result<T, Error>;

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "{msg}"),
            Error::Eof => write!(f, "unexpected end of input"),
            Error::TryFromInt(e) => write!(f, "invalid integer: {e}"),
            Error::ListType(expected, got) => {
                write!(f, "expected type in list: {expected:?}, got: {got:?}")
            }
            Error::MapKey(got) => {
                write!(f, "expected a string for map key, got: {got:?}")
            }
            Error::CannotSerializeNone => {
                write!(
          f,
          "cannot serialize `None` or `()` (use `#[serde(skip_serializing_if = \"Option::is_none\")]`)"
        )
            }
            Error::Enum => {
                write!(f, "enums are not supported")
            }
        }
    }
}

impl std::error::Error for Error {}
