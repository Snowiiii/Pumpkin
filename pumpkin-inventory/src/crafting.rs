use itertools::Itertools;
use pumpkin_registry::{IngredientSlot, IngredientType, Recipe, RecipeResult, ITEM_TAGS, RECIPES};
use pumpkin_world::item::{get_item_protocol_id, ItemStack, ITEMS};

fn check_ingredient_type(ingredient_type: &IngredientType, input: ItemStack) -> bool {
    match ingredient_type {
        IngredientType::Tag(tag) => {
            if let Some(tags) = ITEM_TAGS.get(tag) {
                for tag in tags {
                    if get_item_protocol_id(tag) == input.item_id {
                        return true;
                    }
                }
            }
            false
        }
        IngredientType::Item(item) => get_item_protocol_id(item) == input.item_id,
    }
}

pub fn check_if_matches_crafting(input: [[Option<ItemStack>; 3]; 3]) -> Option<ItemStack> {
    dbg!(input.iter().flatten().collect_vec());
    for recipe in RECIPES.iter() {
        let patterns = recipe.pattern();
        if patterns
            .iter()
            .all(|pattern| pattern.iter().flatten().all(|slot| slot.is_none()))
        {
            continue;
        }
        for pattern in patterns {
            if pattern.iter().enumerate().all(|(i, row)| {
                row.iter()
                    .enumerate()
                    .all(|(j, item)| match (item, input[i][j]) {
                        (Some(item), Some(input)) => match item {
                            IngredientSlot::Single(ingredient) => {
                                check_ingredient_type(ingredient, input)
                            }
                            IngredientSlot::Many(ingredients) => ingredients
                                .iter()
                                .any(|ingredient| check_ingredient_type(ingredient, input)),
                        },
                        (None, None) => true,
                        (Some(_), None) | (None, Some(_)) => false,
                    })
            }) {
                return match recipe.result() {
                    RecipeResult::Single { id } => Some(ItemStack {
                        item_id: get_item_protocol_id(id),
                        item_count: 1,
                    }),
                    RecipeResult::Many { id, count } => Some(ItemStack {
                        item_id: get_item_protocol_id(id),
                        item_count: *count,
                    }),
                    RecipeResult::Special => None,
                };
            }
        }
    }
    None
}
