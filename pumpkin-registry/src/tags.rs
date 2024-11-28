use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::sync::LazyLock;

use crate::IngredientType;

#[derive(Deserialize, Eq, PartialEq, Hash)]
pub enum TagCategory {
    #[serde(rename = "instrument")]
    Instrument,
    #[serde(rename = "worldgen/biome")]
    WorldGenBiome,
    #[serde(rename = "point_of_interest_type")]
    PointOfInterest,
    #[serde(rename = "entity_type")]
    Entity,
    #[serde(rename = "damage_type")]
    DamageType,
    #[serde(rename = "banner_pattern")]
    BannerPattern,
    #[serde(rename = "block")]
    Block,
    #[serde(rename = "fluid")]
    Fluid,
    #[serde(rename = "enchantment")]
    Enchantment,
    #[serde(rename = "cat_variant")]
    Cat,
    #[serde(rename = "painting_variant")]
    Painting,
    #[serde(rename = "item")]
    Item,
    #[serde(rename = "game_event")]
    GameEvent,
}

pub static TAGS: LazyLock<HashMap<TagCategory, HashMap<String, Vec<TagType>>>> =
    LazyLock::new(|| {
        let mut map = HashMap::new();
        let tags_str = include_str!("../../assets/tags.json");
        let tags: Vec<TagCollection> =
            serde_json::from_str(tags_str).expect("Valid tag collections");
        for tag in tags {
            map.insert(tag.name, tag.values);
        }
        map
    });

pub fn get_tag_values(tag_category: TagCategory, tag: &str) -> Option<&Vec<TagType>> {
    TAGS.get(&tag_category)
        .expect("Should deserialize all tag categories")
        .get(tag)
}

#[derive(Deserialize)]
pub struct TagCollection {
    name: TagCategory,
    #[serde(flatten)]
    values: HashMap<String, Vec<TagType>>,
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
        impl Visitor<'_> for TagVisitor {
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

#[cfg(test)]
mod test {
    use crate::tags::TAGS;

    #[test]
    // This test assures that all tags that exist are loaded into the tags registry
    fn load_tags() {
        assert!(!TAGS.is_empty())
    }
}
