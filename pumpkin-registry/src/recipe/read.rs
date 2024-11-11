use crate::flatten_3x3;
use crate::recipe::read::ingredients::{IngredientSlot, Ingredients};
use crate::recipe::read::SpecialCraftingType::{
    ArmorDye, BannerDuplicate, BookCloning, Firework, RepairItem, ShieldDecoration,
    ShulkerboxColoring, SuspiciousStew, TippedArrow,
};
use crate::recipe::recipe_formats::{ShapedCrafting, ShapelessCrafting};
use serde::de::{Error, MapAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::str::FromStr;

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

impl RecipeType {
    pub const fn is_shapeless(&self) -> bool {
        // I have not checked exactly which ones require shape and which don't!
        !matches!(self, Self::Crafting(CraftingType::Shaped))
    }
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
            "crafting_transmute" => Ok(Crafting(Transmute)),
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
    use serde::de::{Error, SeqAccess, Visitor};
    use serde::{de, Deserialize, Deserializer};
    use std::collections::HashMap;
    use std::fmt::Formatter;
    use std::hash::Hash;

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
        type Value = IngredientType;
        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            write!(formatter, "valid item type")
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match v.strip_prefix('#') {
                Some(tag) => Ok(IngredientType::Tag(tag.to_string())),
                None => Ok(IngredientType::Item(v.to_string())),
            }
        }
    }
    impl<'de> Deserialize<'de> for IngredientType {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_str(IngredientTypeVisitor)
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
                IngredientSlot::Many(ingredients) => ingredients.contains(other),
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

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    Ok(IngredientSlot::Single(IngredientTypeVisitor.visit_str(v)?))
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

    pub struct Ingredients(pub Vec<IngredientSlot>);
    impl<'de> Deserialize<'de> for Ingredients {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct IngredientsVisitor;
            impl<'de> Visitor<'de> for IngredientsVisitor {
                type Value = Ingredients;
                fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                    write!(formatter, "valid ingredients")
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

#[derive(Debug)]
pub enum RecipeResult {
    Many {
        count: u8,
        id: String,
        // TODO
        components: Option<serde_json::Value>,
    },
    Single {
        id: String,
        // TODO
        components: Option<serde_json::Value>,
    },
    Special,
}

impl RecipeResult {
    pub fn id(&self) -> &str {
        match self {
            Self::Many { id, .. } | Self::Single { id, .. } => id,
            Self::Special => "minecraft:air",
        }
    }
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
            Components,
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
                let mut components: Option<serde_json::Value> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Fields::Id => visit_option(&mut map, &mut id, "id")?,
                        Fields::Count => visit_option(&mut map, &mut count, "count")?,
                        Fields::Components => {
                            visit_option(&mut map, &mut components, "components")?
                        }
                    }
                }

                let id = id
                    .ok_or_else(|| de::Error::missing_field("id"))?
                    .to_string();
                if let Some(count) = count {
                    Ok(RecipeResult::Many {
                        id,
                        count,
                        components,
                    })
                } else {
                    Ok(RecipeResult::Single { id, components })
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(RecipeResult::Single {
                    id: v.to_string(),
                    components: None,
                })
            }
        }

        // Evaluate putting type constraint on RecipeResult, because only Crafting Transmute can call visit_str
        deserializer.deserialize_any(ResultVisitor)
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

            // Transmute
            Input,
            Material,
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
                let mut transmute_input: Option<IngredientSlot> = None;
                let mut transmute_material: Option<IngredientSlot> = None;
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
                        Fields::Input => visit_option(&mut map, &mut transmute_input, "input"),
                        Fields::Material => {
                            visit_option(&mut map, &mut transmute_material, "material")
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
                    RecipeType::Crafting(CraftingType::Shapeless) => {
                        let ingredients =
                            ingredients.ok_or_else(|| de::Error::missing_field("ingredients"))?;
                        Ok(Recipe::from(ShapelessCrafting::new(ingredients.0, result)))
                    }
                    RecipeType::Crafting(CraftingType::Special(_)) => Ok(Recipe::from(Test {
                        recipe_type,
                        result,
                    })),
                    RecipeType::Crafting(CraftingType::DecoratedPot) => Ok(Recipe::from(Test {
                        recipe_type,
                        result,
                    })),
                    RecipeType::Crafting(CraftingType::Transmute) => {
                        let _input =
                            transmute_input.ok_or_else(|| de::Error::missing_field("input"))?;
                        // Maybe also has material
                        Ok(Recipe::from(Test {
                            recipe_type,
                            result,
                        }))
                    }
                    RecipeType::Smithing(_) => Ok(Recipe::from(Test {
                        recipe_type,
                        result: RecipeResult::Special,
                    })),
                    _ => Ok(Recipe::from(Test {
                        recipe_type,
                        result,
                    })),
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
            "input",
            "material",
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
        None => {
            *option = Some(map.next_value()?);
            Ok(())
        }
    }
}

pub struct Recipe {
    pub recipe_type: RecipeType,
    pattern: Vec<[[Option<IngredientSlot>; 3]; 3]>,
    result: RecipeResult,
}

impl Recipe {
    pub fn pattern(&self) -> &[[[Option<IngredientSlot>; 3]; 3]] {
        &self.pattern
    }

    pub fn result(&self) -> &RecipeResult {
        &self.result
    }

    pub fn implemented(&self) -> bool {
        match self.recipe_type {
            RecipeType::Crafting(crafting_type) => {
                matches!(
                    crafting_type,
                    CraftingType::Shapeless | CraftingType::Shaped
                )
            }
            _ => false,
        }
    }
}

struct Test {
    recipe_type: RecipeType,
    result: RecipeResult,
}
impl RecipeTrait for Test {
    fn recipe_type(&self) -> RecipeType {
        self.recipe_type
    }

    fn pattern(&self) -> Vec<[[Option<IngredientSlot>; 3]; 3]> {
        vec![[const { [const { None }; 3] }; 3]]
    }

    fn result(self) -> RecipeResult {
        self.result
    }
}
impl<T: RecipeTrait> From<T> for Recipe {
    fn from(recipe_type: T) -> Self {
        recipe_type.to_recipe()
    }
}

pub trait RecipeTrait: Sized {
    fn recipe_type(&self) -> RecipeType;

    fn pattern(&self) -> Vec<[[Option<IngredientSlot>; 3]; 3]>;

    fn result(self) -> RecipeResult;

    fn to_recipe(self) -> Recipe {
        Recipe {
            recipe_type: self.recipe_type(),
            pattern: self.pattern().into_iter().map(flatten_3x3).collect(),
            result: self.result(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CraftingType {
    Shapeless,
    Shaped,
    Special(SpecialCraftingType),
    DecoratedPot,
    Transmute,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpecialCraftingType {
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
pub enum FireworkCrafting {
    Rocket,
    Star,
    StarFade,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MapCrafting {
    Extending,
    Cloning,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmithingType {
    Normal,
    Trim,
    Transform,
}
