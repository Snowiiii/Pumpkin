use pumpkin_world::item::Item;

#[allow(dead_code)]
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

    /// Set the contents of an item in a slot
    ///
    /// ## Slot
    /// The slot according to https://wiki.vg/Inventory#Player_Inventory
    ///
    /// ## Item
    /// The optional item to place in the slot
    ///
    /// ## Item allowed override
    /// An override, which when enabled, makes it so that invalid items, can be placed in slots they normally can't.
    /// Useful functionality for plugins in the future.
    pub fn set_slot(&mut self, slot: usize, item: Option<Item>, item_allowed_override: bool) {
        match slot {
            0 => {
                // TODO: Add crafting check here
                self.crafting_output = item
            }
            1..=4 => self.crafting[slot - 1] = item,
            5..=8 => {
                match item {
                    None => self.armor[slot - 4] = None,
                    Some(item) => {
                        // TODO: Replace asserts with error handling
                        match slot - 5 {
                            0 => {
                                assert!(item.is_helmet() || item_allowed_override);
                                self.armor[0] = Some(item);
                            }
                            1 => {
                                assert!(item.is_chestplate() || item_allowed_override);
                                self.armor[1] = Some(item)
                            }
                            2 => {
                                assert!(item.is_leggings() || item_allowed_override);
                                self.armor[2] = Some(item);
                            }
                            3 => {
                                assert!(item.is_boots() || item_allowed_override);
                                self.armor[3] = Some(item)
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
            9..=44 => {
                self.items[slot - 9] = item;
            }
            45 => {
                self.offhand = item;
            }
            _ => unreachable!(),
        }
    }

    pub fn set_selected(&mut self, slot: usize) {
        assert!((0..9).contains(&slot));
        self.selected = slot;
    }

    pub fn held_item(&self) -> Option<&Item> {
        debug_assert!((0..9).contains(&self.selected));
        self.items[self.selected + 36 - 9].as_ref()
    }

    pub fn slots(&self) -> Vec<Option<&Item>> {
        let mut slots = vec![self.crafting_output.as_ref()];
        slots.extend(self.crafting.iter().map(|c|c.as_ref()));
        slots.extend(self.armor.iter().map(|c|c.as_ref()));
        slots.extend(self.items.iter().map(|c|c.as_ref()));
        slots.push(self.offhand.as_ref());
        slots
    }
}
