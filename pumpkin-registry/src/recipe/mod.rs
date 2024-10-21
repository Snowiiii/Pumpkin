mod read;
mod recipe_formats;

pub use read::{
    ingredients::IngredientSlot, ingredients::IngredientType, Recipe, RecipeResult, RecipeType,
};
use std::sync::LazyLock;

pub static RECIPES: LazyLock<Vec<Recipe>> = LazyLock::new(|| {
    let mut recipes = vec![];
    for recipe in std::fs::read_dir("assets/recipes").unwrap() {
        let r = recipe.unwrap();
        let s = std::fs::read_to_string(r.path()).unwrap();
        let recipe = serde_json::from_str::<Recipe>(&s).unwrap();
        recipes.push(recipe);
    }

    recipes
});
