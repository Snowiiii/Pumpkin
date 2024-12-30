use super::super::recipe::RecipeType;
use super::read::{
    ingredients::IngredientSlot, CraftingType, RecipeKeys, RecipeResult, RecipeTrait,
};
pub struct ShapedCrafting {
    keys: RecipeKeys,
    pattern: [[Option<char>; 3]; 3],
    output: RecipeResult,
}
impl RecipeKeys {
    fn pattern_to_thing(
        &self,
        pattern: [[Option<char>; 3]; 3],
    ) -> [[Option<IngredientSlot>; 3]; 3] {
        pattern
            .map(|row| row.map(|maybe_char| maybe_char.and_then(|char| self.0.get(&char).cloned())))
    }
}
impl ShapedCrafting {
    pub fn new(keys: RecipeKeys, pattern: [[Option<char>; 3]; 3], output: RecipeResult) -> Self {
        Self {
            keys,
            pattern,
            output,
        }
    }
}

impl RecipeTrait for ShapedCrafting {
    fn recipe_type(&self) -> RecipeType {
        RecipeType::Crafting(CraftingType::Shaped)
    }

    fn pattern(&self) -> Vec<[[Option<IngredientSlot>; 3]; 3]> {
        vec![self.keys.pattern_to_thing(self.pattern)]
    }

    fn result(self) -> RecipeResult {
        self.output
    }
}

pub struct ShapelessCrafting {
    ingredients: Vec<IngredientSlot>,
    output: RecipeResult,
}

impl ShapelessCrafting {
    pub(crate) fn new(ingredients: Vec<IngredientSlot>, output: RecipeResult) -> Self {
        Self {
            ingredients,
            output,
        }
    }
}

impl RecipeTrait for ShapelessCrafting {
    fn recipe_type(&self) -> RecipeType {
        RecipeType::Crafting(CraftingType::Shapeless)
    }

    // Iterating over all permutations is cheaper than resolving and iterating over all tags when trying to check if recipe
    // is correct. Otherwise, we would have to backtrack and check for each item in the recipe input, which tags they are inside,
    // and then sort those permutations
    fn pattern(&self) -> Vec<[[std::option::Option<IngredientSlot>; 3]; 3]> {
        vec![
            self.ingredients.clone(), //.permutations(self.ingredients.len())
        ]
        .into_iter()
        .map(|thing| {
            let mut v1 = [const { None }; 3];
            let mut v2 = [const { None }; 3];
            let mut v3 = [const { None }; 3];
            for (i, thing) in thing.into_iter().enumerate() {
                if i < 3 {
                    v1[i] = Some(thing)
                } else if i < 6 {
                    v2[i - 3] = Some(thing)
                } else {
                    v3[i - 6] = Some(thing)
                }
            }

            [v1, v2, v3]
        })
        .collect()
    }

    fn result(self) -> RecipeResult {
        self.output
    }
}
