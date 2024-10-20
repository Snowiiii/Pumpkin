use pumpkin_registry::{IngredientSlot, IngredientType, Recipe, ITEM_TAGS, RECIPES};
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
    for recipe in RECIPES.iter() {
        let patterns = recipe.pattern();
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
                        (Some(_), None) => false,
                        (None, Some(_)) => false,
                    })
            }) {
                dbg!(recipe.result().clone());
                return Some(ItemStack {
                    item_count: 1,
                    item_id: 0,
                });
            }
        }
    }
    None
}
