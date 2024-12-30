use pumpkin_registry::{
    flatten_3x3, get_tag_values, IngredientSlot, IngredientType, RecipeResult, TagCategory, RECIPES,
};
use pumpkin_world::item::item_registry::get_item;
use pumpkin_world::item::ItemStack;
use rayon::prelude::*;

#[inline(always)]
fn check_ingredient_type(ingredient_type: &IngredientType, input: ItemStack) -> bool {
    match ingredient_type {
        IngredientType::Tag(tag) => {
            let items = match get_tag_values(TagCategory::Item, tag) {
                Some(items) => items,
                None => return false,
            };
            items
                .iter()
                .any(|tag| check_ingredient_type(&tag.to_ingredient_type(), input))
        }
        IngredientType::Item(item) => get_item(item).is_some_and(|item| item.id == input.item_id),
    }
}

pub fn check_if_matches_crafting(input: [[Option<ItemStack>; 3]; 3]) -> Option<ItemStack> {
    let input = flatten_3x3(input);
    RECIPES
        .par_iter()
        .find_any(|recipe| {
            let patterns = recipe.pattern();
            if patterns
                .iter()
                .flatten()
                .flatten()
                .all(|slot| slot.is_none())
            {
                false
            } else if recipe.recipe_type.is_shapeless() {
                shapeless_crafting_match(input, recipe.pattern())
            } else {
                patterns.par_iter().any(|pattern| {
                    pattern.iter().enumerate().all(|(i, row)| {
                        row.iter()
                            .enumerate()
                            .all(|(j, item)| match (item, input[i][j]) {
                                (Some(item), Some(input)) => ingredient_slot_check(item, input),
                                (None, None) => true,
                                (Some(_), None) | (None, Some(_)) => false,
                            })
                    })
                })
            }
        })
        .map(|recipe| match recipe.result() {
            RecipeResult::Single { id, .. } => Some(ItemStack {
                item_id: get_item(id).unwrap().id,
                item_count: 1,
            }),
            RecipeResult::Many { id, count, .. } => Some(ItemStack {
                item_id: get_item(id).unwrap().id,
                item_count: *count,
            }),
            RecipeResult::Special => None,
        })?
}

fn ingredient_slot_check(recipe_item: &IngredientSlot, input: ItemStack) -> bool {
    match recipe_item {
        IngredientSlot::Single(ingredient) => check_ingredient_type(ingredient, input),
        IngredientSlot::Many(ingredients) => ingredients
            .iter()
            .any(|ingredient| check_ingredient_type(ingredient, input)),
    }
}
fn shapeless_crafting_match(
    input: [[Option<ItemStack>; 3]; 3],
    pattern: &[[[Option<IngredientSlot>; 3]; 3]],
) -> bool {
    let mut pattern: Vec<IngredientSlot> = pattern
        .iter()
        .flatten()
        .flatten()
        .flatten()
        .cloned()
        .collect();
    for item in input.into_iter().flatten().flatten() {
        if let Some(index) = pattern.iter().enumerate().find_map(|(i, recipe_item)| {
            if ingredient_slot_check(recipe_item, item) {
                Some(i)
            } else {
                None
            }
        }) {
            pattern.remove(index);
        } else {
            return false;
        }
    }
    pattern.is_empty()
}
