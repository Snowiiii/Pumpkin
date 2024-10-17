use crate::recipe::read::ingredients::{IngredientSlot, IngredientType, Ingredients};
use crate::recipe::read::SpecialCraftingType::{
    ArmorDye, BannerDuplicate, BookCloning, Firework, RepairItem, ShieldDecoration,
    ShulkerboxColoring, SuspiciousStew, TippedArrow,
};
use crate::recipe::recipe_formats::ShapedCrafting;
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::{write, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Deserialize)]
struct RecipeItem {
    item: String,
}

#[derive(Deserialize)]
struct Shaped {
    key: HashMap<String, RecipeItem>,
    pattern: Vec<String>,
    result: RecipeResult,
}

struct Shapeless {
    ingredients: Vec<RecipeItem>,
    result: RecipeResult,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecipeType {
    Blasting,
    CampfireCooking,
    Crafting(CraftingType),
    Smelting,
    Smithing(SmithingType),
    Smoking,
    StoneCutting,
}

impl FromStr for RecipeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use CraftingType::*;
        use FireworkCrafting::*;
        use MapCrafting::*;
        use RecipeType::*;
        let s = s.trim_start_matches("minecraft:");
        match s {
            "blasting" => Ok(Blasting),
            "campfire_cooking" => Ok(CampfireCooking),
            "crafting_shaped" => Ok(Crafting(Shaped)),
            "crafting_shapeless" => Ok(Crafting(Shapeless)),
            "crafting_special_bookcloning" => Ok(Crafting(Special(BookCloning))),
            "crafting_special_repairitem" => Ok(Crafting(Special(RepairItem))),
            "crafting_special_armordye" => Ok(Crafting(Special(ArmorDye))),
            "crafting_special_firework_rocket" => Ok(Crafting(Special(Firework(Rocket)))),
            "crafting_special_firework_star" => Ok(Crafting(Special(Firework(Star)))),
            "crafting_special_firework_star_fade" => Ok(Crafting(Special(Firework(StarFade)))),
            "crafting_special_suspiciousstew" => Ok(Crafting(Special(SuspiciousStew))),
            "crafting_special_mapextending" => {
                Ok(Crafting(Special(SpecialCraftingType::Map(Extending))))
            }
            "crafting_special_mapcloning" => {
                Ok(Crafting(Special(SpecialCraftingType::Map(Cloning))))
            }
            "crafting_special_shulkerboxcoloring" => Ok(Crafting(Special(ShulkerboxColoring))),
            "crafting_special_bannerduplicate" => Ok(Crafting(Special(BannerDuplicate))),
            "crafting_special_shielddecoration" => Ok(Crafting(Special(ShieldDecoration))),
            "crafting_special_tippedarrow" => Ok(Crafting(Special(TippedArrow))),
            "crafting_decorated_pot" => Ok(Crafting(DecoratedPot)),
            "smelting" => Ok(Smelting),
            "smithing" => Ok(Smithing(SmithingType::Normal)),
            "smithing_trim" => Ok(Smithing(SmithingType::Trim)),
            "smithing_transform" => Ok(Smithing(SmithingType::Transform)),
            "smoking" => Ok(Smoking),
            "stonecutting" => Ok(StoneCutting),
            _ => Err(format!("Could not find recipe with id: \"{s}\"")),
        }
    }
}

mod ingredients {
    use serde::de::{MapAccess, SeqAccess, Visitor};
    use serde::{de, Deserialize, Deserializer};
    use std::fmt::Formatter;

    #[derive(Clone)]
    pub enum IngredientType {
        Item(String),
        Tag(String),
    }

    struct IngredientTypeVisitor;
    impl<'de> Visitor<'de> for IngredientTypeVisitor {
        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            write!(formatter, "valid item type")
        }
        type Value = IngredientType;
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            match map.next_key()? {
                Some("item") => Ok(IngredientType::Item(map.next_value()?)),
                Some("tag") => Ok(IngredientType::Tag(map.next_value()?)),
                Some(s) => Err(de::Error::unknown_field(s, &["item", "tag"])),
                None => Err(de::Error::custom("Is completely empty, very weird")),
            }
        }
    }
    impl<'de> Deserialize<'de> for IngredientType {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(IngredientTypeVisitor)
        }
    }

    pub enum IngredientSlot {
        Single(IngredientType),
        Many(Vec<IngredientType>),
    }

    impl<'de> Deserialize<'de> for IngredientSlot {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct SlotTypeVisitor;
            impl<'de> Visitor<'de> for SlotTypeVisitor {
                fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                    write!(formatter, "valid ingredient slot")
                }

                type Value = IngredientSlot;

                fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
                where
                    A: MapAccess<'de>,
                {
                    Ok(IngredientSlot::Single(
                        IngredientTypeVisitor.visit_map(map)?,
                    ))
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let mut ingredients: Vec<IngredientType> = vec![];
                    while let Some(element) = seq.next_element()? {
                        ingredients.push(element)
                    }
                    if ingredients.len() == 1 {
                        Ok(IngredientSlot::Single(ingredients[0].clone()))
                    } else {
                        Ok(IngredientSlot::Many(ingredients))
                    }
                }
            }
            deserializer.deserialize_any(SlotTypeVisitor)
        }
    }

    pub struct Ingredients(Vec<IngredientSlot>);
    impl<'de> Deserialize<'de> for Ingredients {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct IngredientsVisitor;
            impl<'de> Visitor<'de> for IngredientsVisitor {
                type Value = Ingredients;
                fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                    todo!()
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: SeqAccess<'de>,
                {
                    let mut ingredients = vec![];
                    while let Some(element) = seq.next_element()? {
                        ingredients.push(element)
                    }

                    Ok(Ingredients(ingredients))
                }
            }
            deserializer.deserialize_seq(IngredientsVisitor)
        }
    }
}

pub enum RecipeResult {
    Many { count: u8, id: String },
    Single { id: String },
}

impl<'de> Deserialize<'de> for RecipeResult {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Fields {
            Count,
            Id,
        }
        struct ResultVisitor;
        impl<'de> Visitor<'de> for ResultVisitor {
            type Value = RecipeResult;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "valid recipe result")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut id: Option<&str> = None;
                let mut count: Option<u8> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Fields::Id => visit_option(&mut map, &mut id, "id")?,
                        Fields::Count => visit_option(&mut map, &mut count, "count")?,
                    }
                }

                let id = id
                    .ok_or_else(|| de::Error::missing_field("id"))?
                    .to_string();
                if let Some(count) = count {
                    Ok(RecipeResult::Many { id, count })
                } else {
                    Ok(RecipeResult::Single { id })
                }
            }
        }
        deserializer.deserialize_map(ResultVisitor)
    }
}
pub struct RecipeKeys(HashMap<char, IngredientSlot>);
impl<'de> Deserialize<'de> for RecipeKeys {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyVisitor;
        impl<'de> Visitor<'de> for KeyVisitor {
            type Value = HashMap<char, IngredientSlot>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "existing key inside recipe")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut return_map = HashMap::new();
                while let Some(next) = map.next_key()? {
                    let test: &str = next;
                    let c: char = test.chars().next().unwrap();
                    let ingredient_type: IngredientSlot = map.next_value()?;

                    return_map.insert(c, ingredient_type);
                }
                Ok(return_map)
            }
        }
        deserializer.deserialize_map(KeyVisitor).map(Self)
    }
}
impl<'de> Deserialize<'de> for Recipe {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Fields {
            Type,
            Category,
            Group,
            Key,
            Pattern,
            Result,
            Ingredients,
            Ingredient,

            // Only exists sometimes, at least on Shaped crafting
            #[serde(rename = "show_notification")]
            ShowNotification,

            // Armor
            Addition,
            Base,
            Template,

            // Smelting
            CookingTime,
            Experience,
        }

        struct RecipeVisitor;
        impl<'de> Visitor<'de> for RecipeVisitor {
            type Value = Recipe;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "valid recipe")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut recipe_type: Option<&str> = None;
                let mut category: Option<&str> = None;
                let mut group: Option<&str> = None;
                let mut keys: Option<RecipeKeys> = None;
                let mut pattern: Option<Vec<&str>> = None;
                let mut result: Option<RecipeResult> = None;
                let mut ingredients: Option<Ingredients> = None;
                let mut ingredient: Option<IngredientSlot> = None;
                let mut addition: Option<IngredientSlot> = None;
                let mut base: Option<IngredientSlot> = None;
                let mut template: Option<IngredientSlot> = None;
                let mut cookingtime: Option<u16> = None;
                let mut experience: Option<f32> = None;
                let mut show_notification: Option<bool> = None;
                while let Some(key) = map.next_key()? {
                    (match key {
                        Fields::Type => visit_option(&mut map, &mut recipe_type, "type"),
                        Fields::Group => visit_option(&mut map, &mut group, "group"),
                        Fields::Category => visit_option(&mut map, &mut category, "group"),
                        Fields::Key => visit_option(&mut map, &mut keys, "key"),
                        Fields::Pattern => visit_option(&mut map, &mut pattern, "pattern"),
                        Fields::Result => visit_option(&mut map, &mut result, "result"),
                        Fields::Ingredients => {
                            visit_option(&mut map, &mut ingredients, "ingredients")
                        }
                        Fields::Ingredient => visit_option(&mut map, &mut ingredient, "ingredient"),
                        Fields::Addition => visit_option(&mut map, &mut addition, "addition"),
                        Fields::Base => visit_option(&mut map, &mut base, "base"),
                        Fields::Template => visit_option(&mut map, &mut template, "template"),
                        Fields::CookingTime => {
                            visit_option(&mut map, &mut cookingtime, "cookingtime")
                        }
                        Fields::Experience => visit_option(&mut map, &mut experience, "experience"),
                        Fields::ShowNotification => {
                            visit_option(&mut map, &mut show_notification, "show_notification")
                        }
                    })?
                }

                let recipe_type: RecipeType = recipe_type
                    .ok_or_else(|| de::Error::missing_field("type"))?
                    .parse()
                    .unwrap();
                let result = result.ok_or_else(|| de::Error::missing_field("result"))?;
                match recipe_type {
                    RecipeType::Crafting(CraftingType::Shaped) => {
                        Ok(Recipe::from(ShapedCrafting::new(
                            keys.ok_or_else(|| de::Error::missing_field("keys"))?,
                            pattern
                                .ok_or_else(|| de::Error::missing_field("pattern"))?
                                .into_iter()
                                .map(|s| s.to_string())
                                .collect(),
                            result,
                        )))
                    }
                    _ => Ok(Recipe(Box::new(Test { recipe_type }))),
                }
            }
        }

        const FIELDS: &[&str] = &[
            "type",
            "category",
            "group",
            "key",
            "pattern",
            "result",
            "ingredients",
            "ingredient",
            "addition",
            "base",
            "template",
            "cookingtime",
            "experience",
            "show_notification",
        ];

        deserializer.deserialize_struct("Recipe", FIELDS, RecipeVisitor)
    }
}

#[inline(always)]
fn visit_option<'de, T: Deserialize<'de>, Map: MapAccess<'de>>(
    map: &mut Map,
    option: &mut Option<T>,
    field: &'static str,
) -> Result<(), Map::Error> {
    match option {
        Some(_) => Err(<Map as MapAccess>::Error::duplicate_field(field)),
        None => Ok(*option = Some(map.next_value()?)),
    }
}

struct Recipe(Box<dyn RecipeTrait>);

struct Test {
    recipe_type: RecipeType,
}
impl RecipeTrait for Test {
    fn recipe_type(&self) -> RecipeType {
        self.recipe_type
    }
}
impl<T: RecipeTrait + 'static> From<T> for Recipe {
    fn from(value: T) -> Self {
        Recipe(Box::new(value))
    }
}

pub trait RecipeTrait {
    fn recipe_type(&self) -> RecipeType;
}

impl Deref for Recipe {
    type Target = Box<dyn RecipeTrait>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CraftingType {
    Shapeless,
    Shaped,
    Special(SpecialCraftingType),
    DecoratedPot,
}
#[derive(Debug, Clone, Copy, PartialEq)]
enum SpecialCraftingType {
    BookCloning,
    RepairItem,
    ArmorDye,
    Firework(FireworkCrafting),
    SuspiciousStew,
    ShulkerboxColoring,
    Map(MapCrafting),
    BannerDuplicate,
    ShieldDecoration,
    TippedArrow,
}
#[derive(Debug, Clone, Copy, PartialEq)]
enum FireworkCrafting {
    Rocket,
    Star,
    StarFade,
}
#[derive(Debug, Clone, Copy, PartialEq)]
enum MapCrafting {
    Extending,
    Cloning,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SmithingType {
    Normal,
    Trim,
    Transform,
}

#[cfg(test)]
mod test {
    use crate::recipe::read::{CraftingType, Recipe};
    use crate::recipe::RecipeType;

    #[test]
    fn check_all_recipes() {
        let mut files = std::fs::read_dir("../assets/recipes").unwrap();
        let len = files.count();
        for recipe in std::fs::read_dir("../assets/recipes").unwrap() {
            let r = recipe.unwrap();
            let s = std::fs::read_to_string(r.path()).unwrap();
            let recipe = serde_json::from_str::<Recipe>(&s).unwrap();
            if recipe.recipe_type() == RecipeType::Crafting(CraftingType::Shaped) {}
        }
    }
}
