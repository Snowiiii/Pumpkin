use super::super::recipe::RecipeType;
use super::read::{CraftingType, RecipeKeys, RecipeResult, RecipeTrait};

struct Pattern<const W: usize, const H: usize>([[char; W]; H]);

pub struct ShapedCrafting {
    keys: RecipeKeys,
    pattern: Vec<String>,
    output: RecipeResult,
}

pub struct UnshapedCrafting {}

impl ShapedCrafting {
    pub const fn new(keys: RecipeKeys, pattern: Vec<String>, output: RecipeResult) -> Self {
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
}
