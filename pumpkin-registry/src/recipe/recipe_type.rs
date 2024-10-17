use crate::recipe::read::RecipeType;

struct AbstractItem(u32);

enum Size {
    OneToOne {
        input: AbstractItem,
    },
    Custom3x3 {
        top: [Option<AbstractItem>; 3],
        middle: [Option<AbstractItem>; 3],
        bottom: [Option<AbstractItem>; 3],
    },
    Custom2x2 {
        top: [Option<AbstractItem>; 2],
        bottom: [Option<AbstractItem>; 2],
    },
}

pub trait Recipe {
    const RECIPE_TYPE: RecipeType;
    const CRAFT: Size;
    const OUTPUT: AbstractItem;
}
