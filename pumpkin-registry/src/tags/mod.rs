use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;

mod item;

use crate::IngredientType;
pub use item::ITEM_TAGS;

#[derive(Deserialize)]
pub struct Tag {
    values: Vec<TagType>,
}

#[derive(Clone)]
pub enum TagType {
    Item(String),
    Tag(String),
}

impl TagType {
    pub fn to_ingredient_type(&self) -> IngredientType {
        match self {
            TagType::Tag(tag) => IngredientType::Tag(tag.to_string()),
            TagType::Item(item) => IngredientType::Item(item.to_string()),
        }
    }
}

impl<'de> Deserialize<'de> for TagType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TagVisitor;
        impl<'de> Visitor<'de> for TagVisitor {
            type Value = TagType;
            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "valid tag")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v.strip_prefix('#') {
                    Some(v) => Ok(TagType::Tag(v.to_string())),
                    None => Ok(TagType::Item(v.to_string())),
                }
            }
        }
        deserializer.deserialize_str(TagVisitor)
    }
}
