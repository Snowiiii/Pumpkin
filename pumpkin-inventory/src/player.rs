use pumpkin_world::item::Item;

pub struct PlayerInventory {
    // Main Inventory + Hotbar
    crafting: [Option<Item>; 4],
    crafting_output: Option<Item>,
    items: [Option<Item>; 36],
    armor: [Option<Item>; 4],
    offhand: Option<Item>,
    // current selected slot in hotbar
    selected: usize,
}

pub struct Hotbar<'a>(&'a mut [Option<Item>;9]);

impl Hotbar<'_> {
    fn get_mut(&mut self, index: usize) -> &mut Option<Item> {
        &mut self.0[index]
    }
}

pub struct Armor<'a>(&'a mut [Option<Item>; 4]);



impl Default for PlayerInventory {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerInventory {
    pub fn new() -> Self {
        Self {
            crafting: [None; 4],
            crafting_output: None,
            items: [None; 36],
            armor: [None; 4],
            offhand: None,
            // TODO: What when player spawns in with an different index ?
            selected: 0,
        }
    }

    pub fn set_slot(slot: u32, item: Item) {}

    pub fn set_selected(&mut self, slot: i16) {
        assert!((0..9).contains(&slot));
        self.selected = slot;
    }
}
