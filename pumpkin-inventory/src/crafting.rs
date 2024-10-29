use pumpkin_registry::{
    flatten_3x3, IngredientSlot, IngredientType, RecipeResult, ITEM_TAGS, RECIPES,
};
use pumpkin_world::item::{get_item_protocol_id, ItemStack};

fn check_ingredient_type(ingredient_type: &IngredientType, input: ItemStack) -> bool {
    match ingredient_type {
        IngredientType::Tag(tag) => {
            let tag = tag.strip_prefix("minecraft:").unwrap();
            dbg!(tag);
            let items = ITEM_TAGS.get(tag).expect("tag to be a valid");
            items
                .iter()
                .any(|tag| check_ingredient_type(&tag.to_ingredient_type(), input))
        }
        IngredientType::Item(item) => get_item_protocol_id(item) == input.item_id,
    }
}

pub fn check_if_matches_crafting(input: [[Option<ItemStack>; 3]; 3]) -> Option<ItemStack> {
    let input = flatten_3x3(input);
    for recipe in RECIPES.iter() {
        let patterns = recipe.pattern();
        if patterns
            .iter()
            .flatten()
            .flatten()
            .all(|slot| slot.is_none())
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
