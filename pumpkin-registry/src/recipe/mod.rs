mod read;
mod recipe_formats;
mod recipe_type;

use crate::recipe::read::CraftingType;
use read::RecipeResult;
pub use read::{ingredients::IngredientSlot, ingredients::IngredientType, Recipe, RecipeType};
use std::cell::LazyCell;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

pub static RECIPES: LazyLock<Vec<Recipe>> = LazyLock::new(|| {
    let mut recipes = vec![];
    for recipe in std::fs::read_dir("../assets/recipes").unwrap() {
        let r = recipe.unwrap();
        let s = std::fs::read_to_string(r.path()).unwrap();
        let recipe = serde_json::from_str::<Recipe>(&s).unwrap();
        recipes.push(recipe);
    }

    recipes
});
