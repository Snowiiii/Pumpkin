mod read;
mod recipe_formats;

pub use read::{
    ingredients::IngredientSlot, ingredients::IngredientType, Recipe, RecipeResult, RecipeType,
};
use std::sync::LazyLock;

pub fn flatten_3x3<T: std::marker::Copy>(input: [[Option<T>; 3]; 3]) -> [[Option<T>; 3]; 3] {
    let mut final_output = [[None; 3]; 3];

    let mut row_alignment = 0;
    let mut column_alignment = 2;

    for row in input {
        let mut row_values = [false; 3];
        for (i, item) in row.iter().enumerate().take(column_alignment + 1) {
            if item.is_some() {
                row_values[i] = true;
                if i < column_alignment {
                    column_alignment = i;
                }
            }
        }
        if row_values.iter().all(|val| *val) {
            row_alignment += 1;
        }
    }

    for (i, row) in &mut final_output.iter_mut().enumerate() {
        let input_row = input.get(i + row_alignment);

        for (j, item) in row.iter_mut().enumerate() {
            *item = match input_row.and_then(|val| val.get(j + column_alignment)) {
                None => None,
                Some(None) => None,
                Some(Some(val)) => Some(*val),
            }
        }
    }

    final_output
}
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
