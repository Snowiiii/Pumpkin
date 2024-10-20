use crate::recipe::read::ingredients::{IngredientSlot, Ingredients};
use crate::recipe::read::SpecialCraftingType::{
    ArmorDye, BannerDuplicate, BookCloning, Firework, RepairItem, ShieldDecoration,
    ShulkerboxColoring, SuspiciousStew, TippedArrow,
};
use crate::recipe::recipe_formats::ShapedCrafting;
use itertools::Itertools;
use serde::de::{Error, MapAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Formatter;
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
pub mod ingredients {
    use serde::de::{MapAccess, SeqAccess, Visitor};
    use serde::{de, Deserialize, Deserializer};
    use std::collections::HashMap;
    use std::fmt::Formatter;
    use std::hash::{Hash, Hasher};

    #[derive(Clone, PartialEq, Debug, Eq, Hash)]
    pub enum IngredientType {
        Item(String),
        Tag(String),
    }

    impl IngredientType {
        pub fn to_all_types(&self, item_tags: &HashMap<String, Vec<String>>) -> Vec<String> {
            match &self {
                IngredientType::Tag(tag) => item_tags.get(tag).unwrap().clone(),
                IngredientType::Item(s) => vec![s.to_string()],
            }
        }
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

    #[derive(Clone, Debug, Eq, PartialEq, Hash)]
    pub enum IngredientSlot {
        Single(IngredientType),
        Many(Vec<IngredientType>),
    }

    impl PartialEq<IngredientType> for IngredientSlot {
        fn eq(&self, other: &IngredientType) -> bool {
            match self {
                IngredientSlot::Single(ingredient) => other == ingredient,
                IngredientSlot::Many(ingredients) => ingredients.contains(&other),
            }
        }
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

#[derive(Clone, Debug)]
pub enum RecipeResult {
    Many { count: u8, id: String },
    Single { id: String },
    Special,
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
pub struct RecipeKeys(pub(super) HashMap<char, IngredientSlot>);

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

                let result = match recipe_type {
                    RecipeType::Crafting(CraftingType::Special(_))
                    | RecipeType::Crafting(CraftingType::DecoratedPot)
                    | RecipeType::Smithing(_) => RecipeResult::Special,
                    _ => result.ok_or_else(|| de::Error::missing_field("result"))?,
                };

                match recipe_type {
                    RecipeType::Crafting(CraftingType::Shaped) => {
                        let mut rows = [[None; 3], [None; 3], [None; 3]];
                        pattern
                            .ok_or_else(|| de::Error::missing_field("pattern"))?
                            .into_iter()
                            .map(|s| {
                                let mut chars = [None; 3];
                                s.chars()
                                    .enumerate()
                                    .for_each(|(i, char)| chars[i] = Some(char));
                                chars
                            })
                            .enumerate()
                            .for_each(|(i, row)| rows[i] = row);
                        let keys = keys.ok_or_else(|| de::Error::missing_field("keys"))?;
                        Ok(Recipe::from(ShapedCrafting::new(keys, rows, result)))
                    }
                    RecipeType::Crafting(CraftingType::Shapeless) => Ok(Recipe(Box::new(Test {
                        recipe_type,
                        result,
                    }))),
                    RecipeType::Crafting(CraftingType::Special(_)) => Ok(Recipe(Box::new(Test {
                        recipe_type,
                        result,
                    }))),
                    RecipeType::Crafting(CraftingType::DecoratedPot) => {
                        Ok(Recipe(Box::new(Test {
                            recipe_type,
                            result,
                        })))
                    }
                    RecipeType::Smithing(smithing_type) => Ok(Recipe(Box::new(Test {
                        recipe_type,
                        result: RecipeResult::Special,
                    }))),
                    _ => Ok(Recipe(Box::new(Test {
                        recipe_type,
                        result,
                    }))),
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

pub struct Recipe(Box<dyn RecipeTrait + 'static + Send + Sync>);

struct Test {
    recipe_type: RecipeType,
    result: RecipeResult,
}
impl RecipeTrait for Test {
    fn recipe_type(&self) -> RecipeType {
        self.recipe_type
    }

    fn pattern(&self) -> Vec<[[Option<IngredientSlot>; 3]; 3]> {
        vec![[
            [const { None }; 3],
            [const { None }; 3],
            [const { None }; 3],
        ]]
    }

    fn result(&self) -> &RecipeResult {
        &self.result
    }
}
impl<T: RecipeTrait + 'static + Sync + Send> From<T> for Recipe {
    fn from(value: T) -> Self {
        Recipe(Box::new(value))
    }
}

pub trait RecipeTrait {
    fn recipe_type(&self) -> RecipeType;

    fn pattern(&self) -> Vec<[[Option<IngredientSlot>; 3]; 3]>;
    /*fn take_items(&self, input: [[&mut Option<IngredientType>;3];3]) {
        for pattern in self.pattern() {

            if pattern.iter().enumerate().all(|(i,pattern)|{
              pattern.iter().enumerate().all(|(j,pattern)|{
                  match (pattern, &input[i][j]) {
                      (None, None) => true,
                      (Some(p), Some(i)) => p==i,
                      _ => false
                  }
              })
            }) {
                input.into_iter().for_each(|row|{
                    row.into_iter().for_each(|slot|{
                        match slot {
                            None => (),
                            Some(item) => {

                            }
                        }
                    })
                })
            }
        }
    }*/

    fn result(&self) -> &RecipeResult;
}

impl Deref for Recipe {
    type Target = Box<dyn RecipeTrait + 'static + Send + Sync>;

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
