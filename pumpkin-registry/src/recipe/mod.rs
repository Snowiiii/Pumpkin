mod read;
mod recipe_formats;

pub use read::{
    ingredients::IngredientSlot, ingredients::IngredientType, Recipe, RecipeResult, RecipeType,
};
use std::sync::LazyLock;

pub fn flatten_3x3<T: Clone>(input: [[Option<T>; 3]; 3]) -> [[Option<T>; 3]; 3] {
    let mut final_output = [const { [const { None }; 3] }; 3];

    let mut row_alignment = 0;
    let mut column_alignment = 2;

    for row in &input {
        let mut row_values = [false; 3];
        for (i, item) in row.iter().enumerate().take(column_alignment + 1) {
            if item.is_some() {
                row_values[i] = true;
                if i < column_alignment {
                    column_alignment = i;
                }
            }
        }
        if row_values.iter().all(|val| !val) {
            row_alignment += 1;
        }
    }

    for (i, row) in &mut final_output.iter_mut().enumerate() {
        let input_row = input.get(i + row_alignment);
        for (j, item) in row.iter_mut().enumerate() {
            let val = input_row.and_then(|val| val.get(j + column_alignment));
            *item = match val {
                None => None,
                Some(None) => None,
                Some(Some(val)) => Some(val.clone()),
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

#[cfg(test)]
mod test {
    use super::flatten_3x3;

    #[test]
    fn row_flatten() {
        let input = [[None; 3], [None; 3], [Some(()), Some(()), Some(())]];
        let out = [[Some(()), Some(()), Some(())], [None; 3], [None; 3]];
        assert_eq!(flatten_3x3(input), out);
    }

    #[test]
    fn column_flatten() {
        let one_row_right = [None, None, Some(())];
        let one_row_left = [Some(()), None, None];
        let input = [one_row_right, one_row_right, one_row_right];
        let output = [one_row_left, one_row_left, one_row_left];
        assert_eq!(flatten_3x3(input), output)
    }

    #[test]
    fn full_flatten() {
        let input = [[None; 3], [None; 3], [None, None, Some(())]];
        let output = [[Some(()), None, None], [None; 3], [None; 3]];
        assert_eq!(flatten_3x3(input), output)
    }
}
